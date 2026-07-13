import type { Components } from "streamdown";
import { cn } from "@/lib/utils";
import { MarkdownCode } from "./MarkdownCode";
import { MarkdownTable } from "./MarkdownTable";

const ACTION_BTN =
  "h-7 w-7 inline-flex items-center justify-center rounded-md text-xs text-muted-foreground hover:text-foreground hover:bg-muted transition-colors duration-[var(--ds-dur-fast)]";

/** Misaka chat markdown overrides (docs/ui/06-markdown-message-tools.md §2–4). */
export const CHAT_MARKDOWN_COMPONENTS: Components = {
  h1: ({ children, className, ...props }) => (
    <h1
      className={cn(
        "mt-6 mb-3 text-2xl font-semibold tracking-tight",
        className
      )}
      {...props}
    >
      {children}
    </h1>
  ),
  h2: ({ children, className, ...props }) => (
    <h2
      className={cn(
        "mt-5 mb-3 text-xl font-semibold tracking-tight",
        className
      )}
      {...props}
    >
      {children}
    </h2>
  ),
  h3: ({ children, className, ...props }) => (
    <h3
      className={cn("mt-4 mb-2 text-lg font-semibold", className)}
      {...props}
    >
      {children}
    </h3>
  ),
  h4: ({ children, className, ...props }) => (
    <h4
      className={cn("mt-3 mb-2 text-base font-semibold", className)}
      {...props}
    >
      {children}
    </h4>
  ),
  p: ({ children, className, ...props }) => (
    <p className={cn("my-3 leading-7", className)} {...props}>
      {children}
    </p>
  ),
  ul: ({ children, className, ...props }) => (
    <ul
      className={cn(
        "my-3 ml-5 list-disc space-y-1.5 marker:text-muted-foreground/60",
        className
      )}
      {...props}
    >
      {children}
    </ul>
  ),
  ol: ({ children, className, ...props }) => (
    <ol
      className={cn(
        "my-3 ml-5 list-decimal space-y-1.5 marker:text-muted-foreground/60",
        className
      )}
      {...props}
    >
      {children}
    </ol>
  ),
  li: ({ children, className, ...props }) => (
    <li className={cn("pl-1.5 leading-7", className)} {...props}>
      {children}
    </li>
  ),
  blockquote: ({ children, className, ...props }) => (
    <blockquote
      className={cn(
        "my-4 border-l-4 border-border py-1 pl-4 text-muted-foreground italic",
        className
      )}
      {...props}
    >
      {children}
    </blockquote>
  ),
  hr: ({ className, ...props }) => (
    <hr className={cn("my-6 border-border/50", className)} {...props} />
  ),
  a: ({ href, children, className, ...props }) => {
    const external = typeof href === "string" && /^https?:/i.test(href);
    return (
      <a
        href={href}
        className={cn(
          "text-primary underline underline-offset-4 decoration-primary/30 hover:decoration-primary",
          className
        )}
        {...(external
          ? { target: "_blank", rel: "noopener noreferrer" }
          : {})}
        {...props}
      >
        {children}
      </a>
    );
  },
  strong: ({ children, className, ...props }) => (
    <strong
      className={cn("font-semibold text-foreground", className)}
      {...props}
    >
      {children}
    </strong>
  ),
  img: ({ className, alt, ...props }) => (
    <img
      alt={alt ?? ""}
      loading="lazy"
      className={cn(
        "my-3 max-w-full rounded-lg border border-border/40",
        className
      )}
      {...props}
    />
  ),
  code: MarkdownCode,
  pre: ({ children }) => <>{children}</>,
  table: MarkdownTable,
  thead: ({ children, className, ...props }) => (
    <thead
      className={cn("border-b border-border/60 bg-muted/40", className)}
      {...props}
    >
      {children}
    </thead>
  ),
  th: ({ children, className, ...props }) => (
    <th
      className={cn("px-3 py-2 text-left text-sm font-medium", className)}
      {...props}
    >
      {children}
    </th>
  ),
  tbody: ({ children, className, ...props }) => (
    <tbody className={className} {...props}>
      {children}
    </tbody>
  ),
  tr: ({ children, className, ...props }) => (
    <tr
      className={cn("border-b border-border/30 last:border-b-0", className)}
      {...props}
    >
      {children}
    </tr>
  ),
  td: ({ children, className, ...props }) => (
    <td className={cn("px-3 py-2 align-top text-sm", className)} {...props}>
      {children}
    </td>
  ),
};

export { ACTION_BTN };
