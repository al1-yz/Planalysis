import { isTauri } from "@tauri-apps/api/core";
import { DatabaseZap } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";

import type { Repositories } from "../domain/repositories";
import {
  getDatabaseStatus,
  type DatabaseStatus,
  type StartupErrorCode,
} from "../infrastructure/tauri/database-status";
import { createTauriRepositories } from "../infrastructure/tauri/repositories";
import { RepositoriesContext } from "./repositories-context";

const EXPLANATIONS: Record<StartupErrorCode, string> = {
  database_too_new:
    "Este banco de dados foi criado por uma versão mais nova do Planalysis. Atualize o aplicativo para continuar. Nenhum dado foi alterado.",
  not_a_planalysis_database:
    "O arquivo encontrado não é um banco de dados do Planalysis. Nenhum dado foi alterado.",
  migration_failed:
    "A atualização do banco de dados falhou e foi desfeita. Seus dados anteriores continuam intactos.",
  database_unavailable:
    "O Planalysis não conseguiu acessar o arquivo do banco de dados. Verifique as permissões da pasta e o espaço em disco.",
};

type GateProps = {
  children: ReactNode;
  /** Injected for tests. Defaults to detecting the Tauri runtime. */
  isDesktop?: () => boolean;
  loadStatus?: () => Promise<DatabaseStatus>;
  createRepositories?: () => Repositories;
};

type GateState =
  | { kind: "loading" }
  | { kind: "ready"; repositories: Repositories }
  | { kind: "failed"; status: DatabaseStatus };

function unreachable(error: unknown): DatabaseStatus {
  return {
    ready: false,
    schemaVersion: null,
    appVersion: "",
    path: "",
    error: {
      code: "database_unavailable",
      message: error instanceof Error ? error.message : String(error),
    },
  };
}

/**
 * Starts the desktop application only once its database is open. When it cannot be opened, shows
 * why and stops: the existing database is never recreated, overwritten or reset. Outside the
 * desktop application (the browser, tests) it renders its children as they are.
 */
export function DatabaseGate({
  children,
  isDesktop = isTauri,
  loadStatus = getDatabaseStatus,
  createRepositories = createTauriRepositories,
}: GateProps) {
  const [desktop] = useState(isDesktop);
  const [state, setState] = useState<GateState>({ kind: "loading" });

  useEffect(() => {
    if (!desktop) {
      return;
    }
    let cancelled = false;
    loadStatus()
      .then((status) => status, unreachable)
      .then((status) => {
        if (cancelled) {
          return;
        }
        setState(
          status.ready
            ? { kind: "ready", repositories: createRepositories() }
            : { kind: "failed", status },
        );
      });
    return () => {
      cancelled = true;
    };
  }, [desktop, loadStatus, createRepositories]);

  if (!desktop) {
    return children;
  }
  if (state.kind === "ready") {
    return (
      <RepositoriesContext.Provider value={state.repositories}>
        {children}
      </RepositoriesContext.Provider>
    );
  }
  if (state.kind === "failed") {
    return <DatabaseError status={state.status} />;
  }
  return (
    <div className="flex min-h-screen items-center justify-center bg-canvas p-6">
      <p role="status" className="text-sm text-ink-muted">
        Abrindo o banco de dados…
      </p>
    </div>
  );
}

function DatabaseError({ status }: { status: DatabaseStatus }) {
  const code = status.error?.code ?? "database_unavailable";
  return (
    <div className="flex min-h-screen items-center justify-center bg-canvas p-6">
      <div
        role="alert"
        className="w-full max-w-lg rounded-lg border border-border bg-surface p-6 text-ink"
      >
        <span className="mb-4 inline-flex size-12 items-center justify-center rounded-full bg-expense-soft text-expense">
          <DatabaseZap className="size-6" aria-hidden />
        </span>
        <h1 className="font-display text-lg font-semibold">
          Não foi possível abrir o banco de dados
        </h1>
        <p className="mt-2 text-sm text-ink-muted">{EXPLANATIONS[code]}</p>
        {status.path ? (
          <p className="mt-4 text-sm text-ink-muted">
            Arquivo: <span className="break-all text-ink">{status.path}</span>
          </p>
        ) : null}
        <p className="mt-4 text-sm text-ink-muted">
          Feche e abra o Planalysis novamente depois de resolver o problema. O Planalysis nunca
          apaga nem recria um banco de dados existente.
        </p>
        {status.error ? (
          <details className="mt-4 text-sm">
            <summary className="cursor-pointer text-ink-muted">Detalhes técnicos</summary>
            <pre className="mt-2 overflow-x-auto rounded-md bg-surface-sunken p-3 text-xs whitespace-pre-wrap text-ink-subtle">
              {status.error.message}
            </pre>
          </details>
        ) : null}
      </div>
    </div>
  );
}
