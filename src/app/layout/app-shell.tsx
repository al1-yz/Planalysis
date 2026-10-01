import { Menu } from "lucide-react";
import { useState } from "react";
import { Outlet } from "react-router";

import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";

import { BrandMark } from "./brand-mark";
import { NavMenu } from "./nav-menu";

/**
 * Responsive application frame.
 * - lg and up: fixed navigation rail on the left.
 * - below lg: top bar with a menu button that opens the same navigation in a drawer.
 */
export function AppShell() {
  const [drawerOpen, setDrawerOpen] = useState(false);

  return (
    <div className="min-h-dvh">
      <a
        href="#conteudo"
        className="sr-only z-50 rounded-md bg-surface text-sm font-medium text-ink shadow-raised focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:px-4 focus:py-2.5"
      >
        Pular para o conteúdo
      </a>

      <div className="fixed inset-y-0 left-0 hidden w-(--pl-sidebar-width) flex-col border-r border-nav-border bg-nav text-nav-ink lg:flex">
        <div className="flex h-16 items-center px-6 text-nav-active-ink">
          <BrandMark />
        </div>
        <nav aria-label="Navegação principal" className="flex min-h-0 flex-1 flex-col">
          <NavMenu />
        </nav>
      </div>

      <header className="sticky top-0 z-40 flex h-(--pl-topbar-height) items-center gap-2 border-b border-border bg-canvas/90 px-2 backdrop-blur-sm sm:px-4 lg:hidden">
        <Sheet open={drawerOpen} onOpenChange={setDrawerOpen}>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon-lg" aria-label="Abrir menu">
              <Menu className="size-5" />
            </Button>
          </SheetTrigger>
          <SheetContent side="left" className="border-r border-nav-border bg-nav text-nav-ink">
            <div className="flex h-16 shrink-0 items-center px-6 text-nav-active-ink">
              <SheetTitle asChild>
                <span>
                  <BrandMark />
                </span>
              </SheetTitle>
              <SheetDescription className="sr-only">Navegação principal</SheetDescription>
            </div>
            <nav aria-label="Navegação principal" className="flex min-h-0 flex-1 flex-col">
              <NavMenu onNavigate={() => setDrawerOpen(false)} />
            </nav>
          </SheetContent>
        </Sheet>
        <span className="font-display text-base font-semibold tracking-tight text-ink">
          Planalysis
        </span>
      </header>

      <main id="conteudo" tabIndex={-1} className="outline-none lg:pl-(--pl-sidebar-width)">
        <div className="mx-auto w-full max-w-(--pl-content-max) px-4 py-6 sm:px-6 sm:py-8 lg:px-10 lg:py-10">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
