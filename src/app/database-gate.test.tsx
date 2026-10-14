import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { Repositories } from "../domain/repositories";
import type { DatabaseStatus, StartupErrorCode } from "../infrastructure/tauri/database-status";
import { createTauriRepositories } from "../infrastructure/tauri/repositories";
import { DatabaseGate } from "./database-gate";
import { useRepositories } from "./repositories-context";

const repositories: Repositories = createTauriRepositories();

const ready: DatabaseStatus = {
  ready: true,
  schemaVersion: 1,
  appVersion: "0.1.0",
  path: "C:\\dados\\planalysis.db",
  error: null,
};

function failed(code: StartupErrorCode): DatabaseStatus {
  return {
    ready: false,
    schemaVersion: null,
    appVersion: "0.1.0",
    path: "C:\\dados\\planalysis.db",
    error: { code, message: `detalhe técnico de ${code}` },
  };
}

function Probe() {
  const provided = useRepositories();
  return <p>{provided === repositories ? "repositórios disponíveis" : "repositórios errados"}</p>;
}

const createRepositories = () => repositories;

describe("DatabaseGate", () => {
  it("renders the app untouched outside the desktop application", () => {
    const loadStatus = vi.fn<() => Promise<DatabaseStatus>>();
    render(
      <DatabaseGate isDesktop={() => false} loadStatus={loadStatus}>
        <p>conteúdo do app</p>
      </DatabaseGate>,
    );
    expect(screen.getByText("conteúdo do app")).toBeInTheDocument();
    expect(loadStatus).not.toHaveBeenCalled();
  });

  it("shows a loading message, then the app with the repositories available", async () => {
    render(
      <DatabaseGate
        isDesktop={() => true}
        loadStatus={() => Promise.resolve(ready)}
        createRepositories={createRepositories}
      >
        <Probe />
      </DatabaseGate>,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Abrindo o banco de dados…");
    expect(await screen.findByText("repositórios disponíveis")).toBeInTheDocument();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it.each([
    ["database_too_new", "criado por uma versão mais nova do Planalysis"],
    ["not_a_planalysis_database", "não é um banco de dados do Planalysis"],
    ["migration_failed", "A atualização do banco de dados falhou"],
    ["database_unavailable", "não conseguiu acessar o arquivo do banco de dados"],
  ] as const)("explains %s in Portuguese and does not start the app", async (code, expected) => {
    render(
      <DatabaseGate
        isDesktop={() => true}
        loadStatus={() => Promise.resolve(failed(code))}
        createRepositories={createRepositories}
      >
        <p>conteúdo do app</p>
      </DatabaseGate>,
    );
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Não foi possível abrir o banco de dados");
    expect(alert).toHaveTextContent(expected);
    expect(alert).toHaveTextContent("C:\\dados\\planalysis.db");
    expect(alert).toHaveTextContent(`detalhe técnico de ${code}`);
    expect(screen.queryByText("conteúdo do app")).not.toBeInTheDocument();
  });

  it("never offers to recreate, reset or delete an existing database", async () => {
    render(
      <DatabaseGate
        isDesktop={() => true}
        loadStatus={() => Promise.resolve(failed("not_a_planalysis_database"))}
        createRepositories={createRepositories}
      >
        <p>conteúdo do app</p>
      </DatabaseGate>,
    );
    const alert = await screen.findByRole("alert");
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(alert).toHaveTextContent("nunca apaga nem recria");
  });

  it("reports an unreachable backend as an unavailable database", async () => {
    render(
      <DatabaseGate
        isDesktop={() => true}
        loadStatus={() => Promise.reject(new Error("ipc quebrado"))}
        createRepositories={createRepositories}
      >
        <p>conteúdo do app</p>
      </DatabaseGate>,
    );
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("não conseguiu acessar o arquivo do banco de dados");
    expect(alert).toHaveTextContent("ipc quebrado");
  });
});
