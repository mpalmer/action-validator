#!/usr/bin/env node
// @ts-check

import chalk from "chalk";
import fs from "fs";
import * as actionValidator from "@action-validator/core";

function usage(exitCode = 0) {
  console.log(
    [
      `${chalk.underline("Usage:")} action-validator <path_to_action_yaml>`,
      "",
      `${chalk.underline("Arguments:")}`,
      "  <path_to_action_yaml>  Input file",
    ].join("\n")
  );

  process.exit(exitCode);
}

const args = process.argv.slice(2);
if (args.length !== 1) {
  usage(1);
}

if (args[0] === "--help" || args[0] === "-h") {
  usage();
}

const pathToActionYaml = args[0];

if (!fs.existsSync(pathToActionYaml)) {
  console.error(`File not found: ${pathToActionYaml}`);
  process.exit(1);
}

const contents = fs.readFileSync(pathToActionYaml, "utf8");
const actionType =
  pathToActionYaml.endsWith("action.yml") ||
  pathToActionYaml.endsWith("action.yaml")
    ? "action"
    : "workflow";

const result = (() => {
  switch (actionType) {
    case "action":
      return actionValidator.validateAction(contents);
    case "workflow":
      return actionValidator.validateWorkflow(contents);
  }
})();

if (result.errors.length > 0) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
