import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { Provider } from "@/components/provider";
import "./global.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains-mono",
});

export const metadata: Metadata = {
  metadataBase: new URL("https://pennyproxy.com"),
  title: {
    template: "%s | Penny",
    default: "Penny - Serverless for your servers",
  },
  description:
    "A reverse proxy that starts your apps on demand and kills them when idle. Perfect for cheap VPS instances with multiple side projects.",
  icons: {
    icon: [
      { url: "/favicon.ico", sizes: "any" },
      { url: "/favicon-16x16.png", sizes: "16x16", type: "image/png" },
      { url: "/favicon-32x32.png", sizes: "32x32", type: "image/png" },
    ],
    apple: "/apple-touch-icon.png",
  },
  manifest: "/site.webmanifest",
  openGraph: {
    type: "website",
    siteName: "Penny",
    title: "Penny - Serverless for your servers",
    description:
      "A reverse proxy that starts your apps on demand and kills them when idle.",
    images: {
      url: "/og/home/image.png",
      width: 1200,
      height: 630,
      alt: "Penny - Serverless for your servers",
    },
  },
  twitter: {
    card: "summary_large_image",
    title: "Penny - Serverless for your servers",
    description:
      "A reverse proxy that starts your apps on demand and kills them when idle.",
    images: {
      url: "/og/home/image.png",
      width: 1200,
      height: 630,
      alt: "Penny - Serverless for your servers",
    },
  },
};

export default function Layout({ children }: LayoutProps<"/">) {
  return (
    <html
      lang="en"
      className={`${inter.variable} ${jetbrainsMono.variable}`}
      suppressHydrationWarning
    >
      <head>
        <meta
          name="theme-color"
          content="#fafafa"
          media="(prefers-color-scheme: light)"
        />
        <meta
          name="theme-color"
          content="#0a0a0a"
          media="(prefers-color-scheme: dark)"
        />
      </head>
      <body className="flex flex-col min-h-screen font-[family-name:var(--font-inter)]">
        <Provider>{children}</Provider>
      </body>
    </html>
  );
}
