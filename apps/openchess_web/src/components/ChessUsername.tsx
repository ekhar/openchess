"use client";

import Form from "next/form";
import { useFormStatus } from "react-dom";
import { useActionState } from "react";
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
  const [state, formAction] = useActionState(uploadGames, initialState);
  const { setUsername } = useBoardContext();

  if (state.success) {
    setUsername(state.username);
  }

  return (
    <Form
      action={formAction}
      className="space-y-4"
      // The next/form component will automatically handle:
      // 1. Prefetching when the form is in view
      // 2. Client-side navigation on submission
      // 3. Progressive enhancement if JavaScript hasn't loaded
    >
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
    </Form>
  );
}
