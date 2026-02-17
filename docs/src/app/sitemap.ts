import type { MetadataRoute } from "next";
import { source } from "@/lib/source";

export const dynamic = "force-static";

const baseUrl = "https://pennyproxy.com";

export default function sitemap(): MetadataRoute.Sitemap {
  const docPages = source.getPages().map((page) => ({
    url: `${baseUrl}${page.url}`,
  }));

  return [
    { url: baseUrl },
    { url: `${baseUrl}/builder` },
    ...docPages,
  ];
}
