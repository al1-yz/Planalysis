export function BrandMark() {
  return (
    <span className="flex items-center gap-2.5">
      <svg viewBox="0 0 32 32" className="size-8 shrink-0" aria-hidden>
        <rect width="32" height="32" rx="9" className="fill-brand" />
        <path
          d="M9 22.5V9.5h7.2a4.6 4.6 0 0 1 0 9.2H9"
          className="fill-none stroke-on-brand"
          strokeWidth="2.6"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <circle cx="21.5" cy="22.5" r="1.9" className="fill-on-brand" />
      </svg>
      <span className="font-display text-[1.05rem] font-semibold tracking-tight">Planalysis</span>
    </span>
  );
}
