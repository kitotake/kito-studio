import { useCallback, useEffect, useState } from "react";
import { isNativeRuntime, nativeApi } from "@/lib/tauri";
import type { RecentProject } from "@/types/project";

/** Liste des projets récents ; se recharge quand `refreshKey` change. */
export function useRecentProjects(refreshKey: string | null) {
  const [items, setItems] = useState<RecentProject[]>([]);
  const [loaded, setLoaded] = useState(false);

  const reload = useCallback(async () => {
    if (!isNativeRuntime()) {
      setLoaded(true);
      return;
    }
    try {
      setItems(await nativeApi.project.recent());
    } catch {
      setItems([]);
    } finally {
      setLoaded(true);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload, refreshKey]);

  const remove = useCallback(
    async (path: string) => {
      await nativeApi.project.removeRecent(path);
      await reload();
    },
    [reload]
  );

  const clear = useCallback(async () => {
    await nativeApi.project.clearRecent();
    await reload();
  }, [reload]);

  return { items, loaded, reload, remove, clear };
}
