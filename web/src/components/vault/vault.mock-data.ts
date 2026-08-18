import type { QAPair } from "@/components/vault/vault.types";

export const VAULT_ITEMS_PER_PAGE = 5;

export const VAULT_MOCK_DATA: QAPair[] = [
  {
    id: 1,
    question: "How do I reset my password?",
    answer:
      "Navigate to Settings > Security, then click 'Change Password'. You'll receive a confirmation email to verify the change.",
    category: "Account",
    date: "2026-04-07",
  },
  {
    id: 2,
    question: "What are the system requirements for the desktop app?",
    answer:
      "Windows 10 or later, macOS 11 or later. Minimum 4GB RAM, 500MB free disk space. Internet connection required for sync.",
    category: "Technical",
    date: "2026-04-06",
  },
  {
    id: 3,
    question: "How do I export my data?",
    answer:
      "Go to Settings > Data & Privacy > Export Data. Select the format (JSON, CSV, or PDF) and click 'Generate Export'. Download link will be emailed within 24 hours.",
    category: "Data",
    date: "2026-04-05",
  },
  {
    id: 4,
    question: "Can I collaborate with team members?",
    answer:
      "Yes, upgrade to Team plan to invite unlimited members. Share workspaces, assign tasks, and track activity in real-time. Each team member gets their own dashboard with customizable permissions. You can create different roles like Admin, Editor, and Viewer to control access levels. Team analytics show individual and collective productivity metrics. Real-time collaboration features include live cursors, instant syncing, and comment threads. You can also set up automated workflows to streamline repetitive tasks across your team.",
    category: "Features",
    date: "2026-04-04",
  },
  {
    id: 5,
    question: "What payment methods do you accept?",
    answer:
      "We accept all major credit cards (Visa, Mastercard, Amex), PayPal, and ACH transfers for annual plans. Cryptocurrency support is coming soon.",
    category: "Billing",
    date: "2026-04-03",
  },
  {
    id: 6,
    question: "How do I integrate with third-party tools?",
    answer:
      "Visit the Integrations page in your dashboard. Connect to 200+ apps including Slack, Google Drive, Notion, and Salesforce via OAuth.",
    category: "Integrations",
    date: "2026-04-02",
  },
  {
    id: 7,
    question: "What security measures are in place to protect my data?",
    answer:
      "We implement industry-leading security protocols including AES-256 encryption at rest and TLS 1.3 for data in transit. All data centers are SOC 2 Type II certified and located in secure facilities with 24/7 monitoring. We perform regular security audits and penetration testing by third-party firms. Two-factor authentication (2FA) is available for all accounts and required for team plans. We maintain complete audit logs of all access and changes to your data. Our infrastructure includes automated backup systems with point-in-time recovery, DDoS protection, and advanced threat detection. We are also GDPR and CCPA compliant, giving you full control over your data with the ability to export or delete it at any time. Our security team monitors for suspicious activity around the clock and we have an incident response plan ready to deploy if needed.",
    category: "Security",
    date: "2026-04-01",
  },
  {
    id: 8,
    question: "Can I cancel my subscription at any time?",
    answer:
      "Yes, you can cancel anytime from your account settings. No questions asked.",
    category: "Billing",
    date: "2026-03-31",
  },
  {
    id: 9,
    question: "How does the mobile app sync with the desktop version?",
    answer:
      "All data syncs automatically in real-time across all your devices using our cloud infrastructure. Changes made on mobile appear instantly on desktop and vice versa. The sync happens in the background, so you do not need to manually trigger it. If you are offline, changes are queued locally and sync automatically when you reconnect. You can also view sync status and resolve conflicts if the same item was edited on multiple devices.",
    category: "Technical",
    date: "2026-03-30",
  },
  {
    id: 10,
    question: "What's included in the free plan?",
    answer:
      "The free plan includes unlimited personal projects, up to 5GB storage, basic integrations with popular apps, and community support via our forums. You get access to all core features including task management, file storage, and basic reporting. Free accounts can invite up to 3 collaborators per project. While you will not have access to premium features like advanced analytics, priority support, or unlimited file versioning, the free plan is perfect for individuals and small projects.",
    category: "Billing",
    date: "2026-03-29",
  },
  {
    id: 11,
    question: "How do I migrate data from a competitor's platform?",
    answer:
      "We offer automated migration tools for most major platforms including Asana, Trello, Monday.com, and ClickUp. Connect your account through Settings > Import Data, select what you want to migrate (tasks, files, comments, etc.), and we handle the rest. The process typically takes 15-30 minutes depending on data volume. For enterprise migrations or custom platforms, our support team can provide white-glove migration assistance.",
    category: "Data",
    date: "2026-03-28",
  },
  {
    id: 12,
    question: "Is there a student discount?",
    answer:
      "Yes. Students and educators get 50% off all paid plans after verification with a .edu email address.",
    category: "Billing",
    date: "2026-03-27",
  },
];
