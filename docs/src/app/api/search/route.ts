import { createSearchAPI } from "fumadocs-core/search/server";
import { source } from "@/lib/source";

export const dynamic = "force-static";
export const revalidate = false;

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
  }),
);

export const { staticGET: GET } = createSearchAPI("advanced", {
  language: "english",
  indexes,
});
