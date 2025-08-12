#!/usr/bin/env perl

use strict;
use warnings;
use IPC::Open3;
use Symbol 'gensym';

# Run cargo run -- --next-fail perl and capture output
my $cmd = "cargo run -- --next-fail perl";

# Create pipes for stdin, stdout, and stderr
my ($stdin, $stdout, $stderr) = (gensym, gensym, gensym);

# Open the process
my $pid = open3($stdin, $stdout, $stderr, $cmd);

# Close stdin since we don't need to send input
close $stdin;

# Read and output stdout, filtering out DEBUG messages
while (my $line = <$stdout>) {
    # Skip DEBUG messages and other verbose output
    next if $line =~ /^DEBUG:/;
    next if $line =~ /^warning:/;
    next if $line =~ /^Finished `/;
    next if $line =~ /^Running `/;
    next if $line =~ /^Test \d+\/\d+:/;
    next if $line =~ /^\r/;  # Skip carriage return lines
    
    print $line;
}

# Read and output stderr, filtering out DEBUG messages
while (my $line = <$stderr>) {
    # Skip DEBUG messages and other verbose output
    next if $line =~ /^DEBUG:/;
    next if $line =~ /^warning:/;
    next if $line =~ /^Finished `/;
    next if $line =~ /^Running `/;
    next if $line =~ /^Test \d+\/\d+:/;
    next if $line =~ /^\r/;  # Skip carriage return lines
    
    print STDERR $line;
}

# Wait for the process to finish and get exit code
waitpid($pid, 0);
my $exit_code = $? >> 8;

# Exit with the same code
exit $exit_code;
