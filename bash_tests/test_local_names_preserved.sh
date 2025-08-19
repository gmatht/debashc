set -ex

mkdir -p tmp

cargo run --bin debashc examples/061_test_local_names_preserved.sh > tmp/output_local_names.txt 2> tmp/output_local_names.err

fgrep 'sub test_math($first_number, $second_number, $operation) {' < tmp/output_local_names.txt
fgrep 'sub test_strings($input_string, $search_pattern, $replacement) {' < tmp/output_local_names.txt
fgrep 'sub test_arrays($array_name, $index, $new_value) {' < tmp/output_local_names.txt

exit 0

function test_math() {
    local first_number=$1
    local second_number=$2
    local operation=$3
--

function test_strings() {
    local input_string=$1
    local search_pattern=$2
    local replacement=$3
--

function test_arrays() {
    local array_name=$1
    local index=$2
    local new_value=$3
--
}

