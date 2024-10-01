"use client";
// app/components/ChessUsername.tsx
import { useFormState, useFormStatus } from "react-dom";
import { uploadGames } from "@/actions/upload-games";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useBoardContext } from "@/context/UsernameContext";

const initialState = {
  success: false,
  numGames: 0,
  username: "",
  error: null,
};

function SubmitButton() {
  const { pending } = useFormStatus();
  return (
    <Button type="submit" disabled={pending}>
      {pending ? "Submitting..." : "Submit"}
    </Button>
  );
}

export default function ChessUsername() {
  const [state, formAction] = useFormState(uploadGames, initialState);
  const { setUsername } = useBoardContext();

  if (state.success) {
    setUsername(state.username);
  }

  return (
    <form action={formAction} className="space-y-4">
      <div>
        <Input
          type="text"
          name="username"
          placeholder="Enter chess.com username"
          required
        />
      </div>
      <SubmitButton />
      {state.success && (
        <div>
          <p>Success Importing {state.numGames} Games</p>
          <p>(Around {state.numGames * 49} Moves)</p>
          <p>Username: {state.username}</p>
        </div>
      )}
      {state.error && <p className="text-red-500">{state.error}</p>}
    </form>
  );
}
