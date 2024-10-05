import React, { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ArrowRight } from "lucide-react";

const PANTONE_301 = "#0067a4";

const Login: React.FC = () => {
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [userInfo, setUserInfo] = useState("");
    
    const handleLogin = async (e: React.FormEvent) => {
        e.preventDefault();
        setUserInfo("");
        
        const KEYCLOAK_URL = import.meta.env.VITE_KEYCLOAK_URL || "";
        const REALM = import.meta.env.VITE_KEYCLOAK_REALM || "";
        const CLIENT_ID = import.meta.env.VITE_KEYCLOAK_CLIENT_ID || "";
        const CLIENT_SECRET = import.meta.env.VITE_KEYCLOAK_CLIENT_SECRET || "";

        const params = new URLSearchParams({
            client_id: CLIENT_ID,
            grant_type: "password",
            username,
            password,
            client_secret: CLIENT_SECRET,
        });

        try {
            const response = await fetch(`${KEYCLOAK_URL}/realms/${REALM}/protocol/openid-connect/token`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/x-www-form-urlencoded",
                },
                body: params,
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error_description || "Login failed");
            }

            const data = await response.json();
            console.log("Login successful:", data);
            
            // Fetch user info using the access token
            await fetchUserInfo(data.access_token, KEYCLOAK_URL, REALM);
        } catch (error) {
            console.error("Login error:", error);
        }
    };

    const fetchUserInfo = async (accessToken: string, keycloakUrl: string, realm: string) => {
        try {
            const userInfoResponse = await fetch(`${keycloakUrl}/realms/${realm}/protocol/openid-connect/userinfo`, {
                method: "GET",
                headers: {
                    "Authorization": `Bearer ${accessToken}`,
                },
            });

            if (!userInfoResponse.ok) {
                throw new Error("Failed to fetch user info");
            }

            const userInfoData = await userInfoResponse.json();
            console.log("User info:", userInfoData);
            setUserInfo(userInfoData);
        } catch (error) {
            console.error("Error fetching user info:", error);
        }
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
                    {!userInfo ? (
                        <form onSubmit={handleLogin} className="space-y-6">
                            <div className="space-y-2">
                                <Label htmlFor="username">Username</Label>
                                <Input
                                    id="username"
                                    type="text"
                                    placeholder="Enter your username"
                                    value={username}
                                    onChange={(e) => setUsername(e.target.value)}
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
                    ) : (
                        <div>
                            <h3 className="text-lg font-semibold mb-2">Welcome, {userInfo}!</h3>
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    );
};

export default Login;