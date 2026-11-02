// Khmer messages for API error codes (`{"error": code}`) shown in forms.
const messages: Record<string, string> = {
	invalid_credentials: 'លេខទូរស័ព្ទ ឬពាក្យសម្ងាត់មិនត្រឹមត្រូវ',
	phone_taken: 'លេខទូរស័ព្ទនេះបានចុះឈ្មោះរួចហើយ',
	email_taken: 'អ៊ីមែលនេះត្រូវបានប្រើរួចហើយ',
	invalid_phone: 'លេខទូរស័ព្ទមិនត្រឹមត្រូវ',
	invalid_email: 'អ៊ីមែលមិនត្រឹមត្រូវ',
	invalid_name: 'សូមបញ្ចូលឈ្មោះពេញ',
	weak_password: 'ពាក្យសម្ងាត់ត្រូវមានយ៉ាងហោចណាស់ ៨ តួអក្សរ',
	wrong_password: 'ពាក្យសម្ងាត់បច្ចុប្បន្នមិនត្រឹមត្រូវ',
	password_mismatch: 'ពាក្យសម្ងាត់មិនដូចគ្នាទេ',
	unauthorized: 'សូមចូលគណនីម្តងទៀត',
	unavailable: 'មិនអាចភ្ជាប់ទៅម៉ាស៊ីនមេបានទេ'
};

export const errorMessage = (code: string) => messages[code] ?? 'មានបញ្ហា សូមព្យាយាមម្តងទៀត';
