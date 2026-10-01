import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

type EmptyStateProps = {
  icon: LucideIcon;
  title: string;
  description?: string;
  action?: ReactNode;
};

export function EmptyState({ icon: Icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center rounded-lg border border-dashed border-border-strong bg-surface px-6 py-12 text-center sm:py-16">
      <span className="mb-4 inline-flex size-12 items-center justify-center rounded-full bg-brand-soft text-brand-ink">
        <Icon className="size-6" aria-hidden />
      </span>
      <h2 className="font-display text-lg font-semibold text-ink">{title}</h2>
      {description ? <p className="mt-1.5 max-w-md text-sm text-ink-muted">{description}</p> : null}
      {action ? <div className="mt-6">{action}</div> : null}
    </div>
  );
}
