import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { OutputRequest, OutputSuggestion } from "../lib/types";

export function useOutputPreview(request: OutputRequest | null): OutputSuggestion[] {
  const [suggestions, setSuggestions] = useState<OutputSuggestion[]>([]);
  const key = request ? JSON.stringify(request) : "";

  useEffect(() => {
    if (!request) {
      setSuggestions([]);
      return;
    }
    let cancelled = false;
    api
      .suggestOutputs(request)
      .then((result) => {
        if (!cancelled) setSuggestions(result);
      })
      .catch(() => {
        if (!cancelled) setSuggestions([]);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);

  return suggestions;
}
