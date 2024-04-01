
import { Navbar as Nav, NavbarBrand, NavbarContent, NavbarItem, Link } from "@nextui-org/react";


const Navbar = () => {
    return (
        <Nav>
            <NavbarBrand>
                <p className="font-bold text-inherit">Abuse Helper</p>
            </NavbarBrand>
            <NavbarContent className="hidden sm:flex gap-4" justify="center">
                <NavbarItem>
                    <Link color="foreground" href="#">
                        Home
                    </Link>
                </NavbarItem>
                <NavbarItem isActive>
                    <Link href="#" aria-current="page">
                        Customers
                    </Link>
                </NavbarItem>
                <NavbarItem>
                    <Link color="foreground" href="#">
                        Email
                    </Link>
                </NavbarItem>
            </NavbarContent>
        </Nav>
    )
};

export default Navbar;