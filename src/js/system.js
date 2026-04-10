const console = require('node:console');
const fs = require('node:fs');
const process = require('node:process');
const childProcess = require('node:child_process');

const git = {
	lsFiles() {
		const output = childProcess.execFileSync('git', ['ls-files', '-z'], {
			encoding: 'utf8',
		});

		return output.split('\0').filter((path) => path.length > 0);
	},
};

module.exports.console = console;
module.exports.fs = fs;
module.exports.process = process;
module.exports.git = git;
