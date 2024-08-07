import { createClient } from "@rspc/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import { test } from "vitest";

import { useEffect } from "react";
import { createReactQueryProxy } from ".";

type NestedProcedures = {
	nested: {
		procedures: {
			one: {
				kind: "query";
				input: string;
				result: number;
				error: boolean;
			};
			two: {
				kind: "mutation";
				input: string;
				result: { id: string; name: string };
				error: { status: "NOT_FOUND" };
			};
			three: {
				kind: "subscription";
				input: string;
				result: { id: string; name: string };
				error: { status: "NOT_FOUND" };
			};
		};
	};
};

const rspc = createReactQueryProxy<NestedProcedures>();

test("proxy", () => {
	const queryClient = new QueryClient();

	function Component() {
		rspc.nested.procedures.one.useQuery("test");

		const mutation = rspc.nested.procedures.two.useMutation();

		useEffect(() => {
			mutation.mutate("bruh");
		}, [mutation.mutate]);

		rspc.nested.procedures.three.useSubscription("value", {
			onData: (d) => {},
		});

		return null;
	}

	const client = createClient<NestedProcedures>();

	render(
		<rspc.Provider client={client} queryClient={queryClient}>
			<QueryClientProvider client={queryClient}>
				<Component />
			</QueryClientProvider>
		</rspc.Provider>,
	);
});

test("utils", () => {
	const queryClient = new QueryClient();

	function Component() {
		rspc.useUtils().nested.procedures.one.fetch("test");

		return null;
	}

	const client = createClient<NestedProcedures>();

	render(
		<rspc.Provider client={client} queryClient={queryClient}>
			<QueryClientProvider client={queryClient}>
				<Component />
			</QueryClientProvider>
		</rspc.Provider>,
	);
});
