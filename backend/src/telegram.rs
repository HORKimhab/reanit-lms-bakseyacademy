//! Telegram channel as file storage: a bot (admin of a private channel) uploads files as channel
//! posts and streams them back. Works with the public Bot API (≤ 50 MB up / ≤ 20 MB down) or a
//! self-hosted `telegram-bot-api --local` server (≤ 2 GB), whose `getFile` returns a path on its
//! own disk that we read directly.
//!
//! File URLs contain the bot token: nothing from here may reach the browser except bytes.

use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::http::StatusCode;
use bytes::Bytes;
use futures_util::{TryStreamExt, stream::BoxStream};
use serde::Deserialize;

/// Bot API client bound to one storage channel.
#[derive(Clone)]
pub struct Telegram {
    api_url: String,
    token: String,
    channel_id: String,
    local: Option<LocalDir>,
    /// Expected `X-Telegram-Bot-Api-Secret-Token` on webhook calls; `None` disables the webhook.
    webhook_secret: Option<String>,
    /// Admins' chat (group/channel) for payment screenshots; falls back to the storage channel.
    admin_chat_id: Option<String>,
    /// `file_id` → (`file_path`, fetched at). Telegram keeps download paths valid ≥ 1 h.
    paths: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    http: reqwest::Client,
}

/// Re-resolve cached `getFile` paths after this long (inside Telegram's 1 h guarantee).
const PATH_TTL: Duration = Duration::from_secs(50 * 60);

/// How a file is posted: videos get a duration and in-app streaming; everything else is a document.
#[derive(Clone, Copy)]
pub enum UploadKind {
    Video,
    Document,
}

/// File bytes ready to relay: `206` with `content_range` for a range request, else `200`.
pub struct FileBody {
    pub status: StatusCode,
    pub content_length: Option<u64>,
    pub content_range: Option<String>,
    pub stream: BoxStream<'static, std::io::Result<Bytes>>,
}

#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    /// The requested range lies outside the file → 416.
    #[error("range not satisfiable")]
    RangeNotSatisfiable,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Telegram {
    pub fn new(api_url: &str, token: String, channel_id: String, local: Option<LocalDir>) -> Self {
        Self {
            api_url: api_url.trim_end_matches('/').to_owned(),
            token,
            channel_id,
            local,
            webhook_secret: None,
            admin_chat_id: None,
            paths: Arc::default(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_webhook_secret(mut self, secret: Option<String>) -> Self {
        self.webhook_secret = secret;
        self
    }

    pub fn with_admin_chat(mut self, chat_id: Option<String>) -> Self {
        self.admin_chat_id = chat_id;
        self
    }

    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }

    /// Where payment screenshots go: the admins' chat, else the storage channel.
    pub fn admin_chat_id(&self) -> &str {
        self.admin_chat_id.as_deref().unwrap_or(&self.channel_id)
    }

    /// Whether a webhook call carries our secret (constant-time compare). False when no secret is set.
    pub fn webhook_authorized(&self, header: Option<&[u8]>) -> bool {
        match (&self.webhook_secret, header) {
            (Some(secret), Some(got)) if secret.len() == got.len() => {
                secret.bytes().zip(got).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
            }
            _ => false,
        }
    }

    /// Largest file worth uploading: 2000 MB through a `--local` server. The public API accepts
    /// 50 MB uploads but only lets bots download ≤ 20 MB, so anything bigger could never be streamed.
    pub fn max_upload_bytes(&self) -> u64 {
        if self.local.is_some() { 2000 * 1024 * 1024 } else { 20 * 1024 * 1024 }
    }

    /// Whether payment screenshots fall back to the storage channel (no admin chat set).
    pub fn proofs_go_to_storage_channel(&self) -> bool {
        self.admin_chat_id.is_none()
    }

    /// Posts a file of `len` bytes to the storage channel, streaming `body` (not buffered in memory).
    pub async fn upload(
        &self,
        kind: UploadKind,
        file_name: &str,
        mime_type: &str,
        len: u64,
        body: reqwest::Body,
    ) -> anyhow::Result<StoredFile> {
        self.upload_to(&self.channel_id, None, kind, file_name, mime_type, len, body).await
    }

    /// [`Self::upload`] to any chat the bot can post in, with an optional caption.
    #[allow(clippy::too_many_arguments)]
    pub async fn upload_to(
        &self,
        chat_id: &str,
        caption: Option<&str>,
        kind: UploadKind,
        file_name: &str,
        mime_type: &str,
        len: u64,
        body: reqwest::Body,
    ) -> anyhow::Result<StoredFile> {
        let (method, field) = match kind {
            UploadKind::Video => ("sendVideo", "video"),
            UploadKind::Document => ("sendDocument", "document"),
        };
        let part = reqwest::multipart::Part::stream_with_length(body, len)
            .file_name(file_name.to_owned())
            .mime_str(mime_type)?;
        let mut form = reqwest::multipart::Form::new().text("chat_id", chat_id.to_owned()).part(field, part);
        if let Some(caption) = caption {
            form = form.text("caption", caption.to_owned());
        }
        if matches!(kind, UploadKind::Video) {
            form = form.text("supports_streaming", "true");
        }
        let res = self.http.post(self.method_url(method)).multipart(form).send().await?;
        parse_sent_message(&res.bytes().await?)
    }

    /// `getFile`: a download path (public API) or an absolute path on the `--local` server's disk.
    /// Telegram keeps public download paths valid for at least an hour.
    pub async fn file_path(&self, file_id: &str) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct File {
            file_path: Option<String>,
        }
        let res = self
            .http
            .post(self.method_url("getFile"))
            .json(&serde_json::json!({ "file_id": file_id }))
            .send()
            .await?;
        let file: File = parse_envelope(&res.bytes().await?)?;
        file.file_path.ok_or_else(|| anyhow::anyhow!("telegram: file has no path (too big for this Bot API server?)"))
    }

    /// [`Self::file_path`] with a cache; `fresh` bypasses it (e.g. after a failed open).
    pub async fn file_path_cached(&self, file_id: &str, fresh: bool) -> anyhow::Result<String> {
        if !fresh {
            let cache = self.paths.lock().expect("path cache lock");
            if let Some((path, _)) = cache.get(file_id).filter(|(_, at)| at.elapsed() < PATH_TTL) {
                return Ok(path.clone());
            }
        }
        let path = self.file_path(file_id).await?;
        self.paths.lock().expect("path cache lock").insert(file_id.to_owned(), (path.clone(), Instant::now()));
        Ok(path)
    }

    /// Opens `file_path` (from [`Self::file_path`]) honoring an HTTP `Range` header.
    pub async fn open(&self, file_path: &str, range: Option<&str>) -> Result<FileBody, OpenError> {
        match &self.local {
            Some(local) => open_local(local, file_path, range).await,
            None => self.open_http(file_path, range).await,
        }
    }
}

/// Where a `--local` Bot API server keeps files, and where that directory is mounted for us.
#[derive(Clone)]
pub struct LocalDir {
    /// The server's `--dir` as it appears in `getFile` paths (e.g. `/var/lib/telegram-bot-api`).
    pub server_dir: String,
    /// The same directory as seen by this process (e.g. a shared Docker volume).
    pub host_dir: PathBuf,
}

/// A file stored as a channel post.
#[derive(Debug, PartialEq)]
pub struct StoredFile {
    pub file_id: String,
    pub file_unique_id: String,
    pub message_id: i64,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub file_name: Option<String>,
    pub duration_seconds: Option<i32>,
}

impl Telegram {
    /// Contains the bot token: never log or return it.
    fn method_url(&self, method: &str) -> String {
        format!("{}/bot{}/{method}", self.api_url, self.token)
    }

    /// Public Bot API: relay Telegram's own (range) response.
    async fn open_http(&self, file_path: &str, range: Option<&str>) -> Result<FileBody, OpenError> {
        let url = format!("{}/file/bot{}/{}", self.api_url, self.token, file_path.trim_start_matches('/'));
        let mut req = self.http.get(url);
        if let Some(range) = range {
            req = req.header(reqwest::header::RANGE, range);
        }
        let res = req.send().await.map_err(anyhow::Error::from)?;
        let status = res.status();
        if status == StatusCode::RANGE_NOT_SATISFIABLE {
            return Err(OpenError::RangeNotSatisfiable);
        }
        if status != StatusCode::OK && status != StatusCode::PARTIAL_CONTENT {
            return Err(anyhow::anyhow!("telegram: file download failed with {status}").into());
        }
        let content_range = res
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        Ok(FileBody {
            status,
            content_length: res.content_length(),
            content_range,
            stream: Box::pin(res.bytes_stream().map_err(std::io::Error::other)),
        })
    }
}

/// `--local` server: read the file from the shared directory.
async fn open_local(local: &LocalDir, file_path: &str, range: Option<&str>) -> Result<FileBody, OpenError> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let path = local_path(file_path, local)
        .ok_or_else(|| anyhow::anyhow!("telegram: file path outside the local Bot API directory"))?;
    let mut file = tokio::fs::File::open(&path).await.map_err(anyhow::Error::from)?;
    let total = file.metadata().await.map_err(anyhow::Error::from)?.len();
    let (status, start, len, content_range) = match parse_range(range, total) {
        Err(()) => return Err(OpenError::RangeNotSatisfiable),
        Ok(None) => (StatusCode::OK, 0, total, None),
        Ok(Some((a, b))) => (StatusCode::PARTIAL_CONTENT, a, b - a + 1, Some(format!("bytes {a}-{b}/{total}"))),
    };
    file.seek(std::io::SeekFrom::Start(start)).await.map_err(anyhow::Error::from)?;
    Ok(FileBody {
        status,
        content_length: Some(len),
        content_range,
        stream: Box::pin(tokio_util::io::ReaderStream::new(file.take(len))),
    })
}

/// `Range: bytes=…` → inclusive `(start, end)` within a file of `total` bytes.
/// `Ok(None)`: no header, or one we ignore (malformed / multi-range) → send the whole file.
/// `Err(())`: syntactically fine but unsatisfiable → 416.
pub fn parse_range(header: Option<&str>, total: u64) -> Result<Option<(u64, u64)>, ()> {
    let Some(spec) = header.and_then(|h| h.trim().strip_prefix("bytes=")) else {
        return Ok(None);
    };
    if spec.contains(',') {
        return Ok(None);
    }
    let Some((start, end)) = spec.split_once('-') else { return Ok(None) };
    let num = |s: &str| -> Option<u64> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) { None } else { s.parse().ok() }
    };

    // bytes=-N: the last N bytes
    if start.is_empty() {
        let Some(n) = num(end) else { return Ok(None) };
        if n == 0 || total == 0 {
            return Err(());
        }
        return Ok(Some((total.saturating_sub(n), total - 1)));
    }

    // bytes=A- / bytes=A-B
    let Some(a) = num(start) else { return Ok(None) };
    let b = match end {
        "" => None,
        e => match num(e) {
            Some(b) if b >= a => Some(b),
            _ => return Ok(None),
        },
    };
    if a >= total {
        return Err(());
    }
    Ok(Some((a, b.map_or(total - 1, |b| b.min(total - 1)))))
}

/// Maps a `--local` server path to our mount; `None` if it is outside `server_dir` or climbs out.
pub fn local_path(file_path: &str, local: &LocalDir) -> Option<PathBuf> {
    let rest = file_path.strip_prefix(local.server_dir.trim_end_matches('/'))?.strip_prefix('/')?;
    let rest = Path::new(rest);
    let plain = rest.components().all(|c| matches!(c, Component::Normal(_)));
    (plain && rest.components().next().is_some()).then(|| local.host_dir.join(rest))
}

/// `sendVideo` / `sendDocument` response body → the stored file.
fn parse_sent_message(body: &[u8]) -> anyhow::Result<StoredFile> {
    let msg: Message = parse_envelope(body)?;
    msg.into_stored_file().ok_or_else(|| anyhow::anyhow!("telegram: message has no file"))
}

/// A video/document posted in `channel` (as configured: numeric id or `@username`), from a webhook
/// `Update` body. `None` for anything else (other chats, text posts, edits, malformed JSON).
pub fn channel_post_file(update: &[u8], channel: &str) -> Option<StoredFile> {
    #[derive(Deserialize)]
    struct Update {
        channel_post: Option<Message>,
    }
    let msg = serde_json::from_slice::<Update>(update).ok()?.channel_post?;
    let chat = msg.chat.as_ref()?;
    let ours = match channel.strip_prefix('@') {
        Some(name) => chat.username.as_deref().is_some_and(|u| u.eq_ignore_ascii_case(name)),
        None => chat.id.to_string() == channel,
    };
    if !ours {
        return None;
    }
    msg.into_stored_file()
}

#[derive(Deserialize)]
struct Message {
    message_id: i64,
    #[serde(default)]
    chat: Option<Chat>,
    video: Option<FileMeta>,
    document: Option<FileMeta>,
}

#[derive(Deserialize)]
struct Chat {
    id: i64,
    username: Option<String>,
}

#[derive(Deserialize)]
struct FileMeta {
    file_id: String,
    file_unique_id: String,
    mime_type: Option<String>,
    file_size: Option<i64>,
    file_name: Option<String>,
    duration: Option<i32>,
}

impl Message {
    fn into_stored_file(self) -> Option<StoredFile> {
        let f = self.video.or(self.document)?;
        Some(StoredFile {
            file_id: f.file_id,
            file_unique_id: f.file_unique_id,
            message_id: self.message_id,
            mime_type: f.mime_type,
            size_bytes: f.file_size,
            file_name: f.file_name,
            duration_seconds: f.duration,
        })
    }
}

/// Bot API `{"ok": true, "result": …}` / `{"ok": false, "description": …}`.
fn parse_envelope<T: serde::de::DeserializeOwned>(body: &[u8]) -> anyhow::Result<T> {
    #[derive(Deserialize)]
    struct Envelope<T> {
        ok: bool,
        result: Option<T>,
        description: Option<String>,
    }
    let env: Envelope<T> = serde_json::from_slice(body)?;
    match (env.ok, env.result) {
        (true, Some(result)) => Ok(result),
        _ => anyhow::bail!("telegram: {}", env.description.unwrap_or_else(|| "request failed".into())),
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        extract::{Multipart, Path as AxPath},
        http::HeaderMap,
        routing::{get, post},
    };
    use super::*;

    const TOKEN: &str = "123:ABC";
    const CONTENT: &[u8] = b"0123456789";

    /// Fake Bot API: sendVideo echoes what it received, getFile answers `videos/file_1.mp4`,
    /// and the file endpoint serves CONTENT (206 for `bytes=2-5`, 416 for `bytes=50-`).
    async fn fake_bot_api() -> String {
        let send_video = post(|mut form: Multipart| async move {
            let (mut chat, mut name, mut size, mut streaming) = (String::new(), String::new(), 0, String::new());
            while let Some(field) = form.next_field().await.unwrap() {
                match field.name().unwrap() {
                    "chat_id" => chat = field.text().await.unwrap(),
                    "supports_streaming" => streaming = field.text().await.unwrap(),
                    "video" => {
                        name = field.file_name().unwrap().to_owned();
                        size = field.bytes().await.unwrap().len();
                    }
                    other => panic!("unexpected field {other}"),
                }
            }
            assert_eq!((chat.as_str(), streaming.as_str()), ("-100777", "true"));
            format!(
                r#"{{"ok":true,"result":{{"message_id":5,"chat":{{"id":-100777}},"date":1,
                   "video":{{"file_id":"FID","file_unique_id":"UID","duration":3,"file_size":{size},
                             "file_name":"{name}","mime_type":"video/mp4"}}}}}}"#
            )
        });
        let app = Router::new()
            .route(&format!("/bot{TOKEN}/sendVideo"), send_video)
            .route(
                &format!("/bot{TOKEN}/getFile"),
                post(|body: String| async move {
                    if body.contains("\"FID\"") {
                        r#"{"ok":true,"result":{"file_id":"FID","file_unique_id":"UID","file_path":"videos/file_1.mp4"}}"#
                    } else {
                        r#"{"ok":false,"error_code":400,"description":"Bad Request: invalid file_id"}"#
                    }
                }),
            )
            .route(
                &format!("/file/bot{TOKEN}/{{*path}}"),
                get(|AxPath(path): AxPath<String>, headers: HeaderMap| async move {
                    assert_eq!(path, "videos/file_1.mp4");
                    match headers.get("range").and_then(|v| v.to_str().ok()) {
                        Some("bytes=2-5") => (
                            StatusCode::PARTIAL_CONTENT,
                            [("content-range", "bytes 2-5/10")],
                            &CONTENT[2..6],
                        ),
                        Some("bytes=50-") => (StatusCode::RANGE_NOT_SATISFIABLE, [("content-range", "bytes */10")], &b""[..]),
                        _ => (StatusCode::OK, [("x-full", "1")], CONTENT),
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        format!("http://{addr}")
    }

    async fn collect(body: FileBody) -> Vec<u8> {
        let chunks: Vec<Bytes> = body.stream.try_collect().await.unwrap();
        chunks.concat()
    }

    #[tokio::test]
    async fn uploads_resolves_and_streams_over_http() {
        let tg = Telegram::new(&fake_bot_api().await, TOKEN.into(), "-100777".into(), None);
        let stored = tg
            .upload(UploadKind::Video, "lesson.mp4", "video/mp4", CONTENT.len() as u64, CONTENT.to_vec().into())
            .await
            .unwrap();
        assert_eq!(
            (stored.file_id.as_str(), stored.message_id, stored.size_bytes, stored.file_name.as_deref()),
            ("FID", 5, Some(10), Some("lesson.mp4"))
        );

        let path = tg.file_path("FID").await.unwrap();
        assert_eq!(path, "videos/file_1.mp4");
        assert!(tg.file_path("nope").await.unwrap_err().to_string().contains("invalid file_id"));

        let full = tg.open(&path, None).await.unwrap();
        assert_eq!((full.status, full.content_length), (StatusCode::OK, Some(10)));
        assert_eq!(collect(full).await, CONTENT);

        let part = tg.open(&path, Some("bytes=2-5")).await.unwrap();
        assert_eq!(
            (part.status, part.content_range.as_deref(), part.content_length),
            (StatusCode::PARTIAL_CONTENT, Some("bytes 2-5/10"), Some(4))
        );
        assert_eq!(collect(part).await, b"2345");

        assert!(matches!(tg.open(&path, Some("bytes=50-")).await, Err(OpenError::RangeNotSatisfiable)));
    }

    #[tokio::test]
    async fn streams_local_server_files_from_disk() {
        let host = std::env::temp_dir().join(format!("tg-local-{}", std::process::id()));
        std::fs::create_dir_all(host.join("123:ABC/videos")).unwrap();
        std::fs::write(host.join("123:ABC/videos/file_1.mp4"), CONTENT).unwrap();
        let local = LocalDir { server_dir: "/var/lib/telegram-bot-api".into(), host_dir: host.clone() };
        let tg = Telegram::new("http://unused.invalid", TOKEN.into(), "-1".into(), Some(local));
        let path = "/var/lib/telegram-bot-api/123:ABC/videos/file_1.mp4";

        let full = tg.open(path, None).await.unwrap();
        assert_eq!((full.status, full.content_length, full.content_range.as_deref()), (StatusCode::OK, Some(10), None));
        assert_eq!(collect(full).await, CONTENT);

        let part = tg.open(path, Some("bytes=-3")).await.unwrap();
        assert_eq!(
            (part.status, part.content_length, part.content_range.as_deref()),
            (StatusCode::PARTIAL_CONTENT, Some(3), Some("bytes 7-9/10"))
        );
        assert_eq!(collect(part).await, b"789");

        assert!(matches!(tg.open(path, Some("bytes=10-")).await, Err(OpenError::RangeNotSatisfiable)));
        assert!(matches!(tg.open("/etc/passwd", None).await, Err(OpenError::Other(_))));
        std::fs::remove_dir_all(host).ok();
    }

    #[test]
    fn ranges() {
        assert_eq!(parse_range(None, 1000), Ok(None));
        assert_eq!(parse_range(Some("bytes=0-99"), 1000), Ok(Some((0, 99))));
        assert_eq!(parse_range(Some("bytes=900-"), 1000), Ok(Some((900, 999))));
        assert_eq!(parse_range(Some("bytes=-100"), 1000), Ok(Some((900, 999))));
        // end past the file is clamped; suffix longer than the file is the whole file
        assert_eq!(parse_range(Some("bytes=990-5000"), 1000), Ok(Some((990, 999))));
        assert_eq!(parse_range(Some("bytes=-5000"), 1000), Ok(Some((0, 999))));
        // ignored → whole file
        for ignored in ["items=0-1", "bytes=a-b", "bytes=5-1", "bytes=0-1,5-6", "bytes=-"] {
            assert_eq!(parse_range(Some(ignored), 1000), Ok(None), "{ignored}");
        }
        // unsatisfiable
        assert_eq!(parse_range(Some("bytes=1000-"), 1000), Err(()));
        assert_eq!(parse_range(Some("bytes=-0"), 1000), Err(()));
        assert_eq!(parse_range(Some("bytes=0-"), 0), Err(()));
    }

    #[test]
    fn local_paths_stay_inside_the_mount() {
        let local = LocalDir { server_dir: "/var/lib/telegram-bot-api".into(), host_dir: "/srv/tg".into() };
        assert_eq!(
            local_path("/var/lib/telegram-bot-api/123:ABC/videos/file_1.mp4", &local),
            Some(PathBuf::from("/srv/tg/123:ABC/videos/file_1.mp4"))
        );
        for bad in [
            "/etc/passwd",
            "/var/lib/telegram-bot-api-other/x",
            "/var/lib/telegram-bot-api/../../etc/passwd",
            "/var/lib/telegram-bot-api/",
            "videos/file_1.mp4",
        ] {
            assert_eq!(local_path(bad, &local), None, "{bad}");
        }
    }

    #[test]
    fn picks_files_posted_in_the_storage_channel() {
        let post = |chat: &str, body: &str| {
            format!(r#"{{"update_id":9,"channel_post":{{"message_id":31,"date":1,"chat":{chat},{body}}}}}"#)
        };
        let ours = r#"{"id":-100777,"type":"channel","username":"reanit_files"}"#;
        let video = r#""video":{"file_id":"V1","file_unique_id":"UV1","duration":90,"mime_type":"video/mp4","file_size":5}"#;
        let doc = r#""document":{"file_id":"D1","file_unique_id":"UD1","file_name":"a.pdf","mime_type":"application/pdf"}"#;

        let got = channel_post_file(post(ours, video).as_bytes(), "-100777").unwrap();
        assert_eq!((got.file_id.as_str(), got.message_id, got.duration_seconds), ("V1", 31, Some(90)));
        assert_eq!(channel_post_file(post(ours, doc).as_bytes(), "@reanit_files").unwrap().file_id, "D1");

        let other = r#"{"id":-100999,"type":"channel"}"#;
        assert_eq!(channel_post_file(post(other, video).as_bytes(), "-100777"), None);
        assert_eq!(channel_post_file(post(ours, r#""text":"hello""#).as_bytes(), "-100777"), None);
        let edited = format!(r#"{{"update_id":9,"edited_channel_post":{{"message_id":31,"date":1,"chat":{ours},{video}}}}}"#);
        assert_eq!(channel_post_file(edited.as_bytes(), "-100777"), None);
        assert_eq!(channel_post_file(b"not json", "-100777"), None);
    }

    #[test]
    fn parses_sent_video_and_document() {
        let video = br#"{"ok":true,"result":{"message_id":42,"chat":{"id":-1001},"date":1,
            "video":{"file_id":"BAAC","file_unique_id":"AgAD","duration":754,"width":1280,"height":720,
                     "mime_type":"video/mp4","file_size":73400320,"file_name":"lesson-1.mp4"}}}"#;
        assert_eq!(
            parse_sent_message(video).unwrap(),
            StoredFile {
                file_id: "BAAC".into(),
                file_unique_id: "AgAD".into(),
                message_id: 42,
                mime_type: Some("video/mp4".into()),
                size_bytes: Some(73_400_320),
                file_name: Some("lesson-1.mp4".into()),
                duration_seconds: Some(754),
            }
        );
        let doc = br#"{"ok":true,"result":{"message_id":7,"chat":{"id":-1001},"date":1,
            "document":{"file_id":"BQAC","file_unique_id":"AgAE","file_name":"book.pdf","mime_type":"application/pdf","file_size":1024}}}"#;
        let stored = parse_sent_message(doc).unwrap();
        assert_eq!((stored.file_id.as_str(), stored.duration_seconds), ("BQAC", None));

        let err = br#"{"ok":false,"error_code":413,"description":"Request Entity Too Large"}"#;
        assert!(parse_sent_message(err).unwrap_err().to_string().contains("Request Entity Too Large"));
        let no_file = br#"{"ok":true,"result":{"message_id":8,"chat":{"id":-1001},"date":1,"text":"hi"}}"#;
        assert!(parse_sent_message(no_file).is_err());
    }
}
