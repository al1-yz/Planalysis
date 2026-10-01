import { Hourglass } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageHeader } from "@/components/common/page-header";

/** Placeholder for sections whose milestone has not been implemented yet. */
export function ComingSoonPage({ title }: { title: string }) {
  return (
    <div className="space-y-8">
      <PageHeader title={title} />
      <EmptyState
        icon={Hourglass}
        title="Em construção"
        description="Esta seção será disponibilizada em uma próxima etapa do Planalysis."
      />
    </div>
  );
}
