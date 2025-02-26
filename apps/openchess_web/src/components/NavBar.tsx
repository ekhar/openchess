import React from "react";
import Link from "next/link";
import { ModeToggle } from "@/components/ModeToggle";
import { Button } from "@/components/ui/button";

const Navbar = () => {
  return (
    <nav className="border-b">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          <div className="flex items-center">
            <Link href="/" className="flex shrink-0 items-center">
              <img
                src="/Mediamodifier-Design.svg"
                alt="OpenChess AI Logo"
                className="h-8 w-8"
              />
              <span className="mr-2 p-2 text-2xl font-bold">OpenChess AI</span>
            </Link>
            <div className="ml-10 hidden md:block">
              <div className="flex items-baseline space-x-4">
                <Button variant="ghost" asChild>
                  <Link href="/about">About</Link>
                </Button>
                <Button variant="ghost" asChild>
                  <Link href="/coming-soon">Coming Soon</Link>
                </Button>
              </div>
            </div>
          </div>
          <div className="flex items-center">
            <ModeToggle />
          </div>
        </div>
      </div>
    </nav>
  );
};

export default Navbar;
