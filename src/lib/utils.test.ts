import { describe, expect, it } from "vitest";

import { cn } from "./utils";

describe("cn", () => {
  it("joins conditional class names", () => {
    const isActive = false;
    expect(cn("px-2", isActive && "hidden", undefined, "text-sm")).toBe("px-2 text-sm");
  });

  it("lets later Tailwind utilities override conflicting earlier ones", () => {
    expect(cn("px-2 py-1", "px-4")).toBe("py-1 px-4");
  });

  it("merges conflicts between custom token colors", () => {
    expect(cn("bg-surface text-ink", "bg-brand")).toBe("text-ink bg-brand");
  });
});
