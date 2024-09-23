"use client";
import React from "react";
import Link from "next/link";
import { ModeToggle } from "@/components/ModeToggle";
import { Button } from "@/components/ui/button";
import { supabase } from "@/utils/supabaseClient"; // Adjust the path as needed

const Navbar = () => {
  // Handler for GitHub Sign-In using Supabase
  const handleGitHubSignIn = async () => {
    const { error } = await supabase.auth.signInWithOAuth({
      provider: "github",
      options: {
        redirectTo: window.location.origin, // Adjust redirect URL if needed
      },
    });
    if (error) {
      console.error("Error signing in with GitHub:", error.message);
    }
  };

  // Handler for Supabase Email Sign-In (if applicable)
  const handleSupabaseSignIn = async () => {
    // Implement your Supabase email sign-in logic here
    // For example, opening a modal or redirecting to a sign-in page
    // This is a placeholder
    console.log("Supabase Email Sign-In clicked");
  };

  return (
    <nav className="border-b">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          <div className="flex items-center">
            <Link href="/" className="flex flex-shrink-0 items-center">
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
          <div className="flex items-center space-x-4">
            {/* Mode Toggle */}
            <ModeToggle />

            {/* Sign In Buttons */}
            <Button variant="outline" onClick={handleGitHubSignIn}>
              Sign In with GitHub
            </Button>
          </div>
        </div>
      </div>
    </nav>
  );
};

export default Navbar;
