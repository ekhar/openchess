// filename: apps/openchess_web/src/app/layout.tsx
import "@/styles/globals.css";
import { GeistSans } from "geist/font/sans";
import { type Metadata } from "next";
import { BoardInfoProvider } from "@/context/UsernameContext";
import { ThemeProvider } from "@/components/theme-provider";
import Navbar from "@/components/NavBar";

export const metadata: Metadata = {
  title: "OpenChessAi",
  description: "Chess Opening AI",
  icons: {
    icon: [{ url: "/Mediamodifier-Design.svg", type: "image/svg+xml" }],
  },
};

type RootLayoutProps = {
  children: React.ReactNode;
};

export default function RootLayout({ children }: Readonly<RootLayoutProps>) {
  return (
    <html lang="en" className={GeistSans.variable} suppressHydrationWarning>
      <body className="min-h-screen bg-background antialiased">
        <BoardInfoProvider>
          <ThemeProvider
            attribute="class"
            defaultTheme="system"
            enableSystem
            disableTransitionOnChange
          >
            <div className="relative flex min-h-screen flex-col">
              <Navbar />
              <main className="flex-1 flex-grow">{children}</main>
            </div>
          </ThemeProvider>
        </BoardInfoProvider>
      </body>
    </html>
  );
}
