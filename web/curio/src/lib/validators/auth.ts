export const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
export const OTP_LENGTH = 6;

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

export function validateOtpDigits(digits: string[]): string | null {
  if (digits.length !== OTP_LENGTH) {
    return "Verification code must be 6 digits.";
  }

  const hasInvalidDigit = digits.some((digit) => !/^\d$/.test(digit));
  if (hasInvalidDigit) {
    return "Verification code must contain only numbers.";
  }

  return null;
}
