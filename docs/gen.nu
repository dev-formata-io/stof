# "nu docs/gen.nu"

# generate docs for a lib by name
def generate [name: string] {
    let source = $"docs/($name).stof"
    let dest = $"docs/out/($name).md"
    stof docs $source $dest
}

generate array
