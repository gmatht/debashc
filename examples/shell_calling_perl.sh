#!/bin/bash

# Example 1: Simple Perl one-liner to print text
echo "=== Example 1: Simple Perl one-liner ==="
perl -e 'print "Hello from Perl!\n"'

# Example 2: Perl script with command line arguments
echo -e "\n=== Example 2: Perl with arguments ==="
perl -e 'foreach $arg (@ARGV) { print "Argument: $arg\n" }' "first" "second" "third"

# Example 3: Perl script processing shell variables
echo -e "\n=== Example 3: Perl processing shell variables ==="
SHELL_VAR="Hello World"
perl -e "print \"Shell variable: $ENV{SHELL_VAR}\n\""

# Example 4: Perl script reading from shell pipeline
echo -e "\n=== Example 4: Perl reading from pipeline ==="
echo "apple\nbanana\ncherry" | perl -ne 'chomp; print "Fruit: $_\n"'

# Example 5: Complex Perl script with here document
echo -e "\n=== Example 5: Perl script with here document ==="
perl << 'EOF'
use strict;
use warnings;

my @numbers = (1, 2, 3, 4, 5);
my $sum = 0;

foreach my $num (@numbers) {
    $sum += $num;
    print "Added $num, sum is now $sum\n";
}

print "Final sum: $sum\n";
EOF
