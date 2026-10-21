import { useRef } from "react";

// While a picked folder waits for its scan to start, events from the scan it replaced are dropped.
export function useReplacedScanFilter() {
  const awaitedRootRef = useRef<string | null>(null);

  const awaitScan = (root: string) => {
    awaitedRootRef.current = root;
  };

  // True when the scan for `root` was still awaited, so its failure is the latest news.
  const abandonScan = (root: string) => {
    const awaited = awaitedRootRef.current === root;
    if (awaited) {
      awaitedRootRef.current = null;
    }
    return awaited;
  };

  const acceptsStart = (root: string) => {
    const awaited = awaitedRootRef.current;
    if (awaited !== null && awaited !== root) {
      return false;
    }
    awaitedRootRef.current = null;
    return true;
  };

  const isReplaced = () => awaitedRootRef.current !== null;

  return { awaitScan, abandonScan, acceptsStart, isReplaced };
}
