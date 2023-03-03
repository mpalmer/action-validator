// @ts-check
import chalk from "chalk";
import * as diff from "diff";
import fs from "fs";
import * as action_validator from "../packages/core/action_validator.js";

const update = process.argv.includes("--update") || process.argv.includes("-u");

const start = Date.now();

const passed = fs
  .readdirSync("test")
  // get all directories in test/
  .map((entry) => `test/${entry}`)
  .filter((entry) => fs.statSync(entry).isDirectory())
  // get the first .yml files in each directory
  .map((testDir) => [testDir, ...fs.readdirSync(testDir)])
  .map(([testDir, ...entries]) => [
    testDir,
    entries.filter((entry) => entry.endsWith(".yml"))[0],
  ])
  // get the test fixture and snapshot files for each directory
  .map(([testDir, testFixtureFileName]) => [
    `${testDir}/${testFixtureFileName}`,
    `${testDir}/validation_state.snap.json`,
  ])
  // read the files
  .map(([testFixtureFile, testSnapshotFile]) => [
    testFixtureFile,
    fs.readFileSync(testFixtureFile, "utf8"),
    testSnapshotFile,
    fs.readFileSync(testSnapshotFile, "utf8"),
  ])
  // validate the test fixture files
  .map(([testFixtureFile, testFixture, testSnapshotFile, oldSnapshot]) => {
    let validationState;

    // treat action.yml files as actions and everything else as workflows
    if (testFixtureFile.endsWith("action.yml")) {
      console.log(
        chalk.bold(` > ${testFixtureFile} `) +
          chalk.bold.gray("(action file)") +
          "\n"
      );
      console.log(chalk.bold.gray("=== OUTPUT START ==="));
      validationState = action_validator.validateAction(testFixture);
      console.log(chalk.bold.gray("=== OUTPUT END ==="));
    } else {
      console.log(
        chalk.bold(` > ${testFixtureFile} `) +
          chalk.bold.gray("(workflow file)") +
          "\n"
      );
      console.log(chalk.bold.gray("=== OUTPUT START ==="));
      validationState = action_validator.validateWorkflow(testFixture);
      console.log(chalk.bold.gray("=== OUTPUT END ==="));
    }

    // stringify the validation state and compare it to the snapshot
    const newSnapshot = JSON.stringify(validationState, null, 2) + "\n";
    const isSnapshotMatch = newSnapshot === oldSnapshot;

    if (isSnapshotMatch) {
      console.log(chalk.bold.green("\n   SNAPSHOT MATCHED"));
    } else {
      // print the diff if the snapshot doesn't match
      console.log(
        chalk.bold[update ? "yellow" : "red"]("\n   SNAPSHOT MISMATCH\n")
      );
      console.log(
        `${chalk.bold.gray("Diff:")} ${chalk.bold.red("red")} ${chalk.gray(
          "for deletions,"
        )} ${chalk.bold.green("green")} ${chalk.gray(
          "for additions,"
        )} ${chalk.bold.gray("grey")} ${chalk.gray("for common parts")}`
      );
      diff.diffJson(oldSnapshot, newSnapshot).forEach((part) => {
        const color = part.added ? "green" : part.removed ? "red" : "grey";
        process.stderr.write(chalk[color](part.value));
      });
      process.stderr.write("\n");
    }

    console.log("\n");

    return [isSnapshotMatch, testFixtureFile, testSnapshotFile, newSnapshot];
  })
  .map(
    update
      ? // update any snapshots that don't match (or create new ones)
        ([isSnapshotMatch, testFixtureFile, testSnapshotFile, newSnapshot]) => {
          if (!isSnapshotMatch) {
            console.log(
              chalk.bold.yellow(`Updating snapshot for ${testFixtureFile}`)
            );

            // @ts-ignore -- no const assertions in JS files =(
            fs.writeFileSync(testSnapshotFile, newSnapshot);
          }

          return true;
        }
      : // check if any snapshots don't match
        ([isSnapshotMatch, testFixtureFile]) => {
          if (!isSnapshotMatch) {
            console.log(
              chalk.bold.red(`Snapshot mismatch for ${testFixtureFile}`)
            );
          }

          return update || isSnapshotMatch;
        }
  )
  // check if all tests passed
  .every((passed) => passed);

// exit with a non-zero exit code if any tests failed
if (!passed) {
  console.log(chalk.bold.red("\nFailed some tests"));

  process.exit(1);
}

const end = Date.now();
console.log(chalk.bold.green(`\nPassed all tests in ${(end - start) / 1000}s`));
