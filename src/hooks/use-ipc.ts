import { useState, useCallback, useRef } from "react";
import { IpcError } from "@/lib/ipc";

interface UseIpcState<T> {
  data: T | null;
  loading: boolean;
  error: IpcError | null;
}

interface UseIpcResult<T, A extends unknown[]> extends UseIpcState<T> {
  execute: (...args: A) => Promise<T>;
  reset: () => void;
}

/**
 * Hook for managing IPC call state (loading, data, error).
 *
 * @example
 * const { data, loading, error, execute } = useIpc(settingsIpc.getAppConfig);
 * useEffect(() => { execute(); }, [execute]);
 */
export function useIpc<T, A extends unknown[]>(
  fn: (...args: A) => Promise<T>
): UseIpcResult<T, A> {
  const [state, setState] = useState<UseIpcState<T>>({
    data: null,
    loading: false,
    error: null,
  });

  const fnRef = useRef(fn);
  fnRef.current = fn;

  const execute = useCallback(async (...args: A): Promise<T> => {
    setState((prev) => ({ ...prev, loading: true, error: null }));

    try {
      const result = await fnRef.current(...args);
      setState({ data: result, loading: false, error: null });
      return result;
    } catch (e) {
      const error =
        e instanceof IpcError ? e : new IpcError("unknown", String(e));
      setState((prev) => ({ ...prev, loading: false, error }));
      throw error;
    }
  }, []);

  const reset = useCallback(() => {
    setState({ data: null, loading: false, error: null });
  }, []);

  return { ...state, execute, reset };
}
