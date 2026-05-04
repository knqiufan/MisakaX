import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon,
} from "lucide-react";
import { Toaster as Sonner, type ToasterProps } from "sonner";
import { useSyncExternalStore } from "react";

function readHtmlTheme(): "light" | "dark" {
  if (typeof document === "undefined") return "light";
  return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

function subscribeToHtmlClass(callback: () => void): () => void {
  const obs = new MutationObserver(callback);
  obs.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
  return () => obs.disconnect();
}

const Toaster = ({ theme: themeProp, ...props }: ToasterProps) => {
  const resolved = useSyncExternalStore(
    subscribeToHtmlClass,
    readHtmlTheme,
    () => "light"
  );
  const theme = themeProp ?? resolved;

  return (
    <Sonner
      theme={theme as NonNullable<ToasterProps["theme"]>}
      className="toaster group"
      toastOptions={{
        classNames: {
          toast:
            "z-[var(--ds-layer-toast)] border border-[color:var(--border-strong)] bg-[color:var(--surface-popover)] text-foreground backdrop-blur-xl shadow-[0_20px_48px_rgba(0,0,0,0.35)] dark:shadow-[0_20px_52px_rgba(0,0,0,0.55)]",
        },
      }}
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
      }}
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border-strong)",
          "--border-radius": "var(--radius-lg)",
        } as React.CSSProperties
      }
      {...props}
    />
  );
};

export { Toaster };
