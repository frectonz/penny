import { createSearchAPI, initAdvancedSearch } from "fumadocs-core/search/server";
import { source } from "@/lib/source";

export const dynamic = "force-dynamic";

const indexes = await Promise.all(
  source.getPages().map(async (page) => {
    const data =
      "load" in page.data && typeof page.data.load === "function"
        ? await page.data.load()
        : page.data;

    const structuredData = data.structuredData ?? {
      headings: [],
      contents: [],
    };

    return {
      id: page.url,
      title: data.title ?? "Untitled",
      description: data.description ?? "",
      url: page.url,
      structuredData,
    };
  })
);

const server = initAdvancedSearch({
  language: "english",
  indexes,
});

const results = await server.search("install");
console.log("Advanced search results:", JSON.stringify(results, null, 2));

export const { GET } = createSearchAPI("advanced", {
  language: "english",
  indexes,
});
