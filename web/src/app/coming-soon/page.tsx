import { Button } from "@/components/ui/button";
import Link from "next/link";

const ComingSoonPage = () => {
  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="mb-6 text-3xl font-bold">
        Exciting Features Coming Soon!
      </h1>

      <section className="mb-8">
        <p className="mb-4">
          I'm working hard to bring you even more powerful tools to enhance your
          chess experience. Here's a sneak peek at what's coming:
        </p>
        <ul className="mb-4 list-inside list-disc space-y-2">
          <li>Compare your games to master-level players by name</li>
          <li>Expansive webscraped database for deeper analysis</li>
          <li>
            Chess GPT: RAG-based chess LLM to answer your opening questions
          </li>
          <li>
            Stockfish integrations with 66 million premade positions from
            Lichess for lightning-fast positional analysis
          </li>
          <li>
            Time-based integrations and statistics to analyze performance with
            move-to-move time variability
          </li>

          <li>
            Chess Blog describing my journey and exploration into more chess
            programming and opening concepts
          </li>

          <li>
            Real time opening chess challenges and multiplayer games for your
            friends and club players
          </li>
        </ul>
      </section>

      <section className="mb-8">
        <h2 className="mb-4 text-2xl font-semibold">Request a Feature</h2>
        <p className="mb-4">
          Have an idea for a feature you'd love to see? We're always eager to
          hear from our users!
        </p>

        <Link href="mailto:ericsrealemail@gmail.com">
          <Button>Submit a Feauture Request</Button>
        </Link>
      </section>

      <section>
        <h2 className="mb-4 text-2xl font-semibold">Stay Tuned!</h2>
        <p>
          I'm excited to bring these features to you soon. Keep an eye on our
          updates, and thank you for your continued support of OpenChess AI!
        </p>
      </section>
    </div>
  );
};

export default ComingSoonPage;
