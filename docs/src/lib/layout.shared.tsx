import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { PennyLogo } from "@/components/penny-logo";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <>
          <PennyLogo size={24} color="#f97316" />
          <span className="font-semibold">Penny</span>
        </>
      ),
      transparentMode: "top",
    },
    githubUrl: "https://github.com/frectonz/penny",
    links: [
      { text: "Documentation", url: "/docs" },
      { text: "Builder", url: "/builder" },
    ],
  };
}
