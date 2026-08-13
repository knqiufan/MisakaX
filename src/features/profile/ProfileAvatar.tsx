import { useTranslation } from "react-i18next";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";

interface ProfileAvatarProps {
  displayName: string;
  avatarUrl?: string | null;
  className?: string;
  fallbackClassName?: string;
}

export function profileInitial(displayName: string, fallback: string): string {
  return Array.from(displayName.trim())[0]?.toLocaleUpperCase() ?? fallback;
}

export function ProfileAvatar({
  displayName,
  avatarUrl,
  className,
  fallbackClassName,
}: ProfileAvatarProps) {
  const { t } = useTranslation("profile");
  const initial = profileInitial(displayName, t("avatarFallback"));

  return (
    <Avatar
      role="img"
      aria-label={t("avatarLabel", { name: displayName })}
      className={cn("shrink-0", className)}
    >
      {avatarUrl ? <AvatarImage src={avatarUrl} alt="" /> : null}
      <AvatarFallback
        className={cn("bg-primary/10 font-medium text-primary", fallbackClassName)}
      >
        {initial}
      </AvatarFallback>
    </Avatar>
  );
}
