"use client";

import { Menu } from "lucide-react";
import { useEffect, useState } from "react";

import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { navLinks } from "@/lib/content";
import { cn } from "@/lib/utils";

export function Nav() {
  const [demoInView, setDemoInView] = useState(false);

  useEffect(() => {
    const demo = document.getElementById("payroll-demo");
    if (!demo) {
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => setDemoInView(entry.isIntersecting),
      {
        rootMargin: "-12% 0px -12% 0px",
        threshold: 0.08,
      },
    );

    observer.observe(demo);
    return () => observer.disconnect();
  }, []);

  return (
    <header className="sticky top-0 z-40 border-b-[0.5px] border-surface-border bg-surface-page/80 backdrop-blur">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-6 lg:px-8">
        <a
          className={cn(
            "text-xl font-bold tracking-tight transition-colors",
            demoInView ? "text-ink-primary" : "text-brand-primary",
          )}
          href="#top"
        >
          Boto
        </a>

        <NavigationMenu className="hidden lg:flex">
          <NavigationMenuList>
            {navLinks.map((link) => (
              <NavigationMenuItem key={link.href}>
                <NavigationMenuLink href={link.href}>
                  {link.label}
                </NavigationMenuLink>
              </NavigationMenuItem>
            ))}
          </NavigationMenuList>
        </NavigationMenu>

        <div className="hidden items-center gap-2 lg:flex">
          <ThemeToggle />
          <Button asChild variant={demoInView ? "outline" : "default"}>
            <a href="/app">Get Started</a>
          </Button>
        </div>

        <div className="flex items-center gap-2 lg:hidden">
          <ThemeToggle />
          <Sheet>
            <SheetTrigger asChild>
              <Button
                aria-label="Open navigation"
                size="icon"
                variant="outline"
              >
                <Menu aria-hidden="true" />
              </Button>
            </SheetTrigger>
            <SheetContent>
              <SheetHeader>
                <SheetTitle>Boto</SheetTitle>
              </SheetHeader>
              <nav className="flex flex-col gap-2 pt-8">
                {navLinks.map((link) => (
                  <SheetClose asChild key={link.href}>
                    <a
                      className="rounded-lg px-2 py-3 text-lg font-semibold text-ink-primary"
                      href={link.href}
                    >
                      {link.label}
                    </a>
                  </SheetClose>
                ))}
              </nav>
              <SheetClose asChild>
                <Button asChild className="mt-auto" variant="outline">
                  <a href="/app">Get Started</a>
                </Button>
              </SheetClose>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
}
