import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';
import * as React from 'react';

import { cn } from '@/lib/utils';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full text-sm font-semibold transition-all duration-200 disabled:pointer-events-none disabled:opacity-55 [&_svg]:pointer-events-none [&_svg]:shrink-0 active:scale-[0.98]',
  {
    variants: {
      variant: {
        primary: 'btn-cta text-white hover:brightness-105',
        secondary: 'bg-secondary text-secondary-foreground hover:brightness-105',
        outline: 'border border-border bg-surface text-ink hover:border-primary-300 hover:bg-primary-50',
        ghost: 'text-ink hover:bg-primary-50',
        soft: 'bg-primary-50 text-primary-700 hover:bg-primary-100',
        dark: 'bg-dark text-white hover:bg-dark/90',
        link: 'h-auto rounded-none p-0 text-primary-600 underline-offset-4 hover:underline',
      },
      size: {
        sm: 'h-9 px-3.5 [&_svg]:size-4',
        md: 'h-11 px-5 [&_svg]:size-[18px]',
        lg: 'h-14 px-7 text-base [&_svg]:size-5',
        icon: 'size-10 [&_svg]:size-5',
      },
    },
    defaultVariants: { variant: 'primary', size: 'md' },
  },
);

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, type, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button';
    return (
      <Comp
        ref={ref}
        type={asChild ? undefined : (type ?? 'button')}
        className={cn(buttonVariants({ variant, size }), className)}
        {...props}
      />
    );
  },
);
Button.displayName = 'Button';

export { Button, buttonVariants };
