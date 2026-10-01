import {
  ArrowLeftRight,
  ChartColumn,
  CreditCard,
  Landmark,
  LayoutDashboard,
  Settings,
  Tags,
  Target,
  Wallet,
  ChartPie,
  type LucideIcon,
} from "lucide-react";

export type NavItem = {
  label: string;
  path: string;
  icon: LucideIcon;
};

export type NavGroup = {
  /** Visible group heading; omitted for the top group. */
  label?: string;
  items: NavItem[];
};

export const primaryNavigation: NavGroup[] = [
  {
    items: [
      { label: "Visão geral", path: "/", icon: LayoutDashboard },
      { label: "Transações", path: "/transacoes", icon: ArrowLeftRight },
    ],
  },
  {
    label: "Organização",
    items: [
      { label: "Contas", path: "/contas", icon: Landmark },
      { label: "Cartões", path: "/cartoes", icon: CreditCard },
      { label: "Categorias", path: "/categorias", icon: Tags },
    ],
  },
  {
    label: "Planejamento",
    items: [
      { label: "Orçamentos", path: "/orcamentos", icon: ChartPie },
      { label: "Metas", path: "/metas", icon: Target },
    ],
  },
  {
    label: "Análise",
    items: [
      { label: "Patrimônio", path: "/patrimonio", icon: Wallet },
      { label: "Relatórios", path: "/relatorios", icon: ChartColumn },
    ],
  },
];

export const footerNavigation: NavItem[] = [
  { label: "Configurações", path: "/configuracoes", icon: Settings },
];
