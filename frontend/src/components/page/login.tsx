import React, { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ArrowRight, ArrowRightCircle } from "lucide-react";

const PANTONE_301 = "#0067a4";

const Login: React.FC = () => {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");

    const handleLogin = (e: React.FormEvent) => {
        e.preventDefault();
        // TODO: Implement actual login logic here
        console.log("Login attempt with:", { email, password });
    };

    return (
        <div className="flex items-center justify-center min-h-screen bg-gray-100">
            <Card className="w-full max-w-md shadow-lg">
                <CardHeader className="space-y-1 text-center pb-8 pt-6">
                    <div className="flex flex-col items-center">
                        <img src="/bell-logo.svg" alt="Bell Logo" className="w-32 h-32 mb-4" />
                        <h2 className="text-2xl font-extrabold tracking-wider" style={{ color: PANTONE_301 }}>
                            ABUSE HELPER
                        </h2>
                    </div>
                    <CardDescription className="mt-2">Please sign in to access the application</CardDescription>
                </CardHeader>
                <CardContent>
                    <form onSubmit={handleLogin} className="space-y-6">
                        <div className="space-y-2">
                            <Label htmlFor="email">Email</Label>
                            <Input
                                id="email"
                                type="email"
                                placeholder="Enter your email"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                required
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="password">Password</Label>
                            <Input
                                id="password"
                                type="password"
                                placeholder="Enter your password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                required
                            />
                        </div>
                        <div className="flex justify-end pt-4">
                            <Button
                                type="submit"
                                className="rounded-full px-6 py-2 flex items-center justify-center text-white transition-colors duration-300 hover:bg-blue-700 group"
                                style={{ backgroundColor: PANTONE_301 }}
                            >
                                <span className="mr-2">Sign In</span>
                                <ArrowRight className="w-5 h-5 transition-transform duration-300 group-hover:translate-x-1" />
                            </Button>
                        </div>
                    </form>
                </CardContent>
            </Card>
        </div>
    );
};

export default Login;
