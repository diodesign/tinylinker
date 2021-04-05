/* itsylinker
 * 
 * Minimalist linker that generates 64-bit RISC-V (RV64I) ELF files
 *
 * Syntax: itsylinker [options] objects...
 * 
 * It accepts the following binutils ld-compatible command-line arguments:
 * 
 * -L <path>        Add <path> to the list of paths that will be searched for the given files to link
 * -o <output>      Generate the linked ELF executable at <output> or a.out in the current working directory if not specified
 * -T <config>      Read linker settings from configuration file <config>
 * --start-group    Mark the start of a group of files in which to resolve all possible references
 * --end-group      Mark the end of a group created by --start-group
 * 
 * --help           Display minimal usage information
 * --version        Display version information
 * 
 * Interspersed in the command line arguments are object and library files to link together to form the final ELF executable.
 * Note: A configuration file must be provided. This is a toml file described in config.rs. It is not compatible with other linkers.
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

extern crate toml;
extern crate serde;
extern crate serde_derive;

mod cmd;     /* command-line parser */
mod context; /* describe the linking context */
mod config;  /* configuration file parser */
mod search;  /* find files for the linking process */

fn main()
{
    let context = cmd::parse_args();
    let config_filename = match context.get_config_file()
    {
        Some(f) => f,
        None =>
        {
            eprintln!("Linker configuration file must be specified with -T");
            std::process::exit(1);
        }
    };

    let config = config::parse_config(&config_filename);
    eprintln!("il: entry symbol = {}", config.get_entry());

    let mut paths = search::Paths::new();

    for item in context.stream_iter()
    {
        match item
        {
            context::StreamItem::SearchPath(f) => paths.add(&f),
            context::StreamItem::Object(f) | context::StreamItem::Archive(f) =>
            {
                match paths.find_file(&f)
                {
                    Some(path) => eprintln!("--> To process: {:?}", path.as_path().to_str()),
                    None =>
                    {
                        eprintln!("Cannot find file {} to link", f);
                        std::process::exit(1);
                    }
                }
            },
            context::StreamItem::Group(g) => for archive in g.iter()
            {
                match archive
                {
                    context::StreamItem::Archive(f) =>
                    {
                        match paths.find_file(&f)
                        {
                            Some(path) => eprintln!("--> (group) To process: {:?}", path.as_path().to_str()),
                            None =>
                            {
                                eprintln!("Cannot find file {} to link", f);
                                std::process::exit(1);
                            }
                        }
                    },
                    _ => eprintln!("??? Unexpected item in group")
                }
            }
        }
    }

    std::process::exit(1);
}