// app/create-game/page.tsx
import { useRouter } from "next/navigation";
import { v4 as uuidv4 } from "uuid";

export default function CreateGamePage() {
  const router = useRouter();

  const handleCreateGame = async () => {
    const gameId = uuidv4();

    // Create a new game entry in the database
    const { data, error } = await fetch("/api/create-game", {
      method: "POST",
      body: JSON.stringify({ gameId }),
    }).then((res) => res.json());

    if (error) {
      console.error(error);
    } else {
      // Redirect to the game page
      router.push(`/game/${gameId}`);
    }
  };

  return (
    <div>
      <h1>Create a New Chess Game</h1>
      <button onClick={handleCreateGame}>Create Game</button>
    </div>
  );
}
