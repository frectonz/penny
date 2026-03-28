import { createSearchAPI } from "fumadocs-core/search/server";
import { source } from "@/lib/source";

export const dynamic = "force-dynamic";

const indexes = await Promise.all(
  source.getPages().map(async (page) => {
    const data =
      "load" in page.data && typeof page.data.load === "function"
        ? await page.data.load()
        : page.data;

    const content =
      data.structuredData?.contents
        ?.map((c: { content: string }) => c.content)
        .join(" ") ?? "";

    return {
      title: data.title ?? "Untitled",
      description: data.description ?? "",
      content,
      url: page.url,
    };
  })
);

export const { GET } = createSearchAPI("simple", {
  language: "english",
  indexes,
});
