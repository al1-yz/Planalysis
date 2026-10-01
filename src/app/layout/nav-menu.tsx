import { NavLink } from "react-router";

import { cn } from "@/lib/utils";

import { footerNavigation, primaryNavigation, type NavItem } from "../navigation";

type NavMenuProps = {
  /** Called after a link is chosen (used by the mobile drawer to close itself). */
  onNavigate?: () => void;
};

/** The navigation list, shared by the desktop sidebar and the mobile drawer. */
export function NavMenu({ onNavigate }: NavMenuProps) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex-1 space-y-6 overflow-y-auto px-3 py-4">
        {primaryNavigation.map((group, index) => (
          <div key={group.label ?? index}>
            {group.label ? (
              <p className="mb-1.5 px-3 text-[0.6875rem] font-semibold tracking-[0.08em] text-nav-ink-muted uppercase">
                {group.label}
              </p>
            ) : null}
            <ul className="space-y-0.5">
              {group.items.map((item) => (
                <li key={item.path}>
                  <NavMenuLink item={item} onNavigate={onNavigate} />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <ul className="space-y-0.5 border-t border-nav-border px-3 py-3">
        {footerNavigation.map((item) => (
          <li key={item.path}>
            <NavMenuLink item={item} onNavigate={onNavigate} />
          </li>
        ))}
      </ul>
    </div>
  );
}

function NavMenuLink({
  item,
  onNavigate,
}: {
  item: NavItem;
  onNavigate: (() => void) | undefined;
}) {
  const Icon = item.icon;
  return (
    <NavLink
      to={item.path}
      end={item.path === "/"}
      onClick={onNavigate}
      className={({ isActive }) =>
        cn(
          "group relative flex h-11 items-center gap-3 rounded-md px-3 text-sm font-medium transition-colors lg:h-10",
          isActive
            ? "bg-nav-active text-nav-active-ink"
            : "text-nav-ink hover:bg-nav-hover hover:text-nav-active-ink",
        )
      }
    >
      {({ isActive }) => (
        <>
          <span
            aria-hidden
            className={cn(
              "absolute inset-y-2 left-0 w-[3px] rounded-r-full bg-nav-indicator transition-opacity",
              isActive ? "opacity-100" : "opacity-0",
            )}
          />
          <Icon
            className={cn(
              "size-[1.125rem]",
              isActive ? "text-nav-indicator" : "text-nav-ink-muted group-hover:text-nav-ink",
            )}
            aria-hidden
          />
          {item.label}
        </>
      )}
    </NavLink>
  );
}
