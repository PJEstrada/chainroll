"use client";

import { motion, useReducedMotion } from "motion/react";
import type * as React from "react";

import { cn } from "@/lib/utils";

type FadeUpProps = {
  children: React.ReactNode;
  className?: string;
  delay?: number;
  mode?: "mount" | "view";
};

export function FadeUp({
  children,
  className,
  delay = 0,
  mode = "view",
}: FadeUpProps) {
  const reducedMotion = useReducedMotion();
  const motionProps = reducedMotion
    ? {}
    : {
        initial: { opacity: 0, y: 14 },
        transition: { duration: 0.2, delay, ease: "easeOut" as const },
      };

  if (mode === "mount") {
    return (
      <motion.div
        animate={reducedMotion ? undefined : { opacity: 1, y: 0 }}
        className={cn(className)}
        {...motionProps}
      >
        {children}
      </motion.div>
    );
  }

  return (
    <motion.div
      className={cn(className)}
      viewport={{ once: true, margin: "-80px" }}
      whileInView={reducedMotion ? undefined : { opacity: 1, y: 0 }}
      {...motionProps}
    >
      {children}
    </motion.div>
  );
}
