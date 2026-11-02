#!/usr/bin/env bash
# End-to-end check of Telegram channel storage: upload a > 20 MB file through the site, then stream
# it back (full + ranged, as a video player seeking would) and compare the bytes.
#
#   SITE=http://localhost:5173 API=http://127.0.0.1:8080 SESSION=<admin session token> scripts/telegram-smoke.sh
#
# SESSION is the value of the `session` cookie of an admin account. Uploads one real file to the
# storage channel (named telegram-smoke-<timestamp>.mp4); delete the channel post afterwards if you like.
set -euo pipefail
: "${SITE:?site URL, e.g. http://localhost:5173}" "${API:?API URL, e.g. http://127.0.0.1:8080}" "${SESSION:?admin session token}"
SIZE_MB=${SIZE_MB:-25}
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
name="telegram-smoke-$(date +%s).mp4"
head -c $((SIZE_MB * 1024 * 1024)) /dev/urandom > "$work/in.mp4"
size=$(wc -c < "$work/in.mp4" | tr -d ' ')
step() { printf '\n== %s\n' "$*"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

step "upload $SIZE_MB MB via $SITE/admin/media/upload"
code=$(curl -sS -o "$work/media.json" -w '%{http_code}' -H "Origin: $SITE" -b "session=$SESSION" \
  -F "size=$size" -F "file=@$work/in.mp4;type=video/mp4;filename=$name" "$SITE/admin/media/upload")
[[ $code == 201 ]] || fail "upload returned $code: $(cat "$work/media.json")"
id=$(sed -E 's/.*"id":"([^"]+)".*/\1/' "$work/media.json"); echo "media $id"

step "signed link from the media library"
url=$(curl -sS -f -H "Authorization: Bearer $SESSION" "$API/api/admin/media?q=$name" \
  | sed -E 's/.*"preview_url":"([^"]+)".*/\1/')
[[ $url == /media/* ]] || fail "no preview_url (is TELEGRAM_* configured?)"

step "full download"
curl -sS -f -o "$work/out.mp4" "$SITE$url"
cmp "$work/in.mp4" "$work/out.mp4" || fail "downloaded bytes differ"
echo "ok: $size bytes identical"

step "seek: Range bytes=$((size - 1048576))-"
code=$(curl -sS -o "$work/tail.bin" -w '%{http_code}' -H "Range: bytes=$((size - 1048576))-" "$SITE$url")
[[ $code == 206 ]] || fail "expected 206, got $code"
cmp "$work/tail.bin" <(tail -c 1048576 "$work/in.mp4") || fail "range bytes differ"
echo "ok: 206 with the last 1 MiB"

step "range past the end"
code=$(curl -sS -o /dev/null -w '%{http_code}' -H "Range: bytes=$((size + 10))-" "$SITE$url")
[[ $code == 416 ]] || fail "expected 416, got $code"
echo "ok: 416"

printf '\nPASS: upload → Telegram → stream (+ seek) for a %s MB file\n' "$SIZE_MB"
