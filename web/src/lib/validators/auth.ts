export const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export function validateEmail(email: string): string | null {
  const value = email.trim();
  if (!value) {
    return "Email is required.";
  }

  if (!EMAIL_REGEX.test(value)) {
    return "Enter a valid email address.";
  }

  return null;
}
