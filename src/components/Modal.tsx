import type { ReactNode } from "react";
import { Modal as HeroModal } from "@heroui/react";

interface ModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  children: ReactNode;
  footer?: ReactNode;
  size?: "xs" | "sm" | "md" | "lg" | "cover" | "full";
  className?: string;
}

export function Modal({
  open,
  onOpenChange,
  title,
  description,
  children,
  footer,
  size = "lg",
  className = "",
}: ModalProps) {
  return (
    <HeroModal.Backdrop
      isOpen={open}
      onOpenChange={onOpenChange}
      variant="opaque"
    >
      <HeroModal.Container placement="center" scroll="inside" size={size}>
        <HeroModal.Dialog className={`app-dialog ${className}`}>
          <HeroModal.CloseTrigger />
          <HeroModal.Header>
            <HeroModal.Heading className="dialog-title">{title}</HeroModal.Heading>
            {description ? <p className="dialog-description">{description}</p> : null}
          </HeroModal.Header>
          <HeroModal.Body className="dialog-body">{children}</HeroModal.Body>
          {footer ? <HeroModal.Footer className="dialog-footer">{footer}</HeroModal.Footer> : null}
        </HeroModal.Dialog>
      </HeroModal.Container>
    </HeroModal.Backdrop>
  );
}
