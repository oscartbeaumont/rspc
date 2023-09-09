// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.
export type Procedures = { queries: { key: "X-Demo-Header"; input: never; result: string; error: Infallible } | { key: "customErr"; input: never; result: null; error: MyCustomError } | { key: "echo"; input: string; result: string; error: Infallible } | { key: "echo2"; input: string; result: string; error: Infallible } | { key: "transformMe"; input: never; result: string; error: Infallible } | { key: "version"; input: never; result: string; error: Infallible }; mutations: { key: "sendMsg"; input: string; result: string; error: Infallible }; subscriptions: never }

export type Infallible = never

export type MyCustomError = "IAmBroke"
