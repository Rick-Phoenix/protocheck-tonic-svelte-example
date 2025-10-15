<script lang="ts">
  import { ConnectError, createClient } from "@connectrpc/connect";
  import { createGrpcWebTransport } from "@connectrpc/connect-web";
  import {
    Greeter,
    type HelloRequest,
    HelloRequestSchema,
  } from "../gen/greeter/greeter_pb";
  import { ViolationsSchema } from "../gen/buf/validate/validate_pb";
  import { createStandardSchema } from "@bufbuild/protovalidate";
  import { create } from "@bufbuild/protobuf";
  import { createForm, type StandardSchemaV1 } from "@tanstack/svelte-form";
  import Input from "$lib/components/ui/input/input.svelte";
  import Button from "$lib/components/ui/button/button.svelte";

  let response = $state("");
  let error = $state("");

  const greetValidator = createStandardSchema(HelloRequestSchema);

  // https://github.com/bufbuild/protovalidate-es/issues/95
  const realValidator: StandardSchemaV1<HelloRequest, HelloRequest> = {
    "~standard": {
      version: 1,
      vendor: "protovalidate-es",
      validate: (value: unknown) => {
        const message = create(HelloRequestSchema, value as HelloRequest);
        return greetValidator["~standard"].validate(message);
      },
    },
  };

  const form = createForm(() => ({
    defaultValues: {
      name: "",
    },
    onSubmit: async ({ value }) => {
      await sayHello(value as HelloRequest);
    },
    validators: {
      onChange: realValidator,
    },
  }));

  async function sayHello(greeting: HelloRequest) {
    try {
      const transport = createGrpcWebTransport({
        baseUrl: "http://localhost:3000",
      });

      const client = createClient(Greeter, transport);
      const res = await client.sayHello(greeting);

      response = res.message;
      error = "";
    } catch (e) {
      if (e instanceof ConnectError) {
        const violations = e.findDetails(ViolationsSchema);
        error = violations[0].violations[0].message;
      }
    }
  }
</script>

<div class="size-full flex-col-center gap-4">
  <h1 class="text-3xl">Tonic + Protocheck + Svelte</h1>
  <form
    class="flex-col-center gap-4"
    onsubmit={(e) => {
      e.preventDefault();
      e.stopPropagation();
      form.handleSubmit();
    }}
  >
    <div class="flex-col-center gap-4">
      <form.Field
        name="name"
      >
        {#snippet children(field)}
          <div class="flex-center gap-4">
            <label for={field.name} class="text-lg">Name: </label>
            <Input
              id={field.name}
              type="text"
              name={field.name}
              value={field.state.value}
              oninput={(e: Event) => {
                const target = e.target as HTMLInputElement;
                field.handleChange(target.value);
              }}
            />
          </div>

          {#if field.state.meta.isTouched && field.state.value.length}
            {#each field.state.meta.errors as error}
              <em>{error!.message}</em>
            {/each}
          {/if}
        {/snippet}
      </form.Field>
    </div>
    <form.Subscribe
      selector={(state) => ({
        canSubmit: state.canSubmit,
        isSubmitting: state.isSubmitting,
      })}
    >
      {#snippet children(state)}
        <Button type="submit" disabled={!state.canSubmit}>
          {state.isSubmitting ? "..." : "Submit"}
        </Button>
      {/snippet}
    </form.Subscribe>
  </form>
  {#if error}
    <h3>Error: {error}</h3>
  {:else if response}
    <h3>{response}</h3>
  {/if}
</div>
