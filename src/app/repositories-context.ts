import { createContext, useContext } from "react";

import type { Repositories } from "../domain/repositories";

export const RepositoriesContext = createContext<Repositories | null>(null);

/** The repositories wired at startup. Only available inside the desktop application. */
export function useRepositories(): Repositories {
  const repositories = useContext(RepositoriesContext);
  if (repositories === null) {
    throw new Error("useRepositories must be used inside the desktop application's DatabaseGate.");
  }
  return repositories;
}
