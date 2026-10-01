import { Compass } from "lucide-react";
import { Link } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { PageHeader } from "@/components/common/page-header";
import { Button } from "@/components/ui/button";

export function NotFoundPage() {
  return (
    <div className="space-y-8">
      <PageHeader title="Página não encontrada" />
      <EmptyState
        icon={Compass}
        title="Não encontramos esta página"
        description="O endereço pode estar incorreto ou a página pode ter sido movida."
        action={
          <Button asChild>
            <Link to="/">Voltar para a visão geral</Link>
          </Button>
        }
      />
    </div>
  );
}
