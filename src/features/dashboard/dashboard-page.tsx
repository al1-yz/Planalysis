import { LayoutDashboard } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageHeader } from "@/components/common/page-header";

export function DashboardPage() {
  return (
    <div className="space-y-8">
      <PageHeader title="Visão geral" description="Resumo das suas finanças no mês." />
      <EmptyState
        icon={LayoutDashboard}
        title="Nenhum dado ainda"
        description="Os resumos do mês aparecerão aqui assim que você cadastrar contas e transações."
      />
    </div>
  );
}
