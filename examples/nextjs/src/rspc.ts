import { httpLink, wsLink, initRspc } from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react";
import { QueryClient } from "@tanstack/react-query";
import type { Procedures } from "../../bindings";

export const client = initRspc<Procedures>({
  links: [
    typeof window === "undefined"
      ? // WebsocketTransport can not be used Server Side, so we provide FetchTransport instead.
        // If you do not plan on using Subscriptions you can use FetchTransport on Client Side as well.
        httpLink({
          url: "http://localhost:4000/rspc",
          batch: true,
        })
      : wsLink({
          url: "ws://localhost:4000/rspc/ws",
        }),
  ],
});

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false, // If you want to retry when requests fail, remove this.
    },
  },
});

export const {
  useContext,
  useMutation,
  useQuery,
  useSubscription,
  Provider: RSPCProvider,
} = createReactQueryHooks<Procedures>(client);
