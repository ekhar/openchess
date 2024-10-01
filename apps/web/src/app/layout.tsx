import "@/styles/globals.css";
import { GeistSans } from "geist/font/sans";
import { type Metadata } from "next";
import { BoardInfoProvider } from "@/context/UsernameContext";
import { ThemeProvider } from "@/components/theme-provider";
import Navbar from "@/components/NavBar";

export const metadata: Metadata = {
  title: "OpenChessAi",
  description: "Chess Opening AI",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" className={`${GeistSans.variable}`}>
      <link rel="icon" href="/Mediamodifier-Design.svg" type="image/svg+xml" />
      <body>
        <BoardInfoProvider>
          <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
            <div className="flex min-h-screen flex-col">
              <Navbar />
              <main className="flex-grow">{children}</main>
            </div>
          </ThemeProvider>
        </BoardInfoProvider>
      </body>
    </html>
  );
}
