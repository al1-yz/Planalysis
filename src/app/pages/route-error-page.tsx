import { TriangleAlert } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { Button } from "@/components/ui/button";

/** Last-resort boundary for unexpected render errors. */
export function RouteErrorPage() {
  return (
    <div className="mx-auto max-w-xl px-4 py-16">
      <title>Erro · Planalysis</title>
      <EmptyState
        icon={TriangleAlert}
        title="Algo deu errado"
        description="Ocorreu um erro inesperado ao exibir esta tela. Seus dados não foram alterados."
        action={<Button onClick={() => window.location.reload()}>Recarregar</Button>}
      />
    </div>
  );
}
