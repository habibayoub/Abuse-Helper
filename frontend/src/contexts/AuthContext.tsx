import React, { createContext, useState, useContext, useEffect, useCallback } from 'react';
import { jwtDecode } from 'jwt-decode';

interface UserInfo {
    name: string;
    username: string;
    email: string;
    role?: string; // Changed from roles?: string[] to role?: string
}

interface DecodedToken {
    exp: number;
    [key: string]: any;
}

interface AuthContextType {
    accessToken: string | null;
    userInfo: UserInfo | null;
    authenticated: boolean;
    login: (accessToken: string, refreshToken: string, userInfo: UserInfo) => void;
    logout: () => Promise<void>;
    refreshAuth: () => Promise<boolean>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const ACCESS_TOKEN_KEY = 'auth_access_token';
const REFRESH_TOKEN_KEY = 'auth_refresh_token';
const USER_INFO_KEY = 'auth_user_info';

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [accessToken, setAccessToken] = useState<string | null>(null);
    const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
    const [authenticated, setAuthenticated] = useState<boolean>(false);

    const login = useCallback((newAccessToken: string, newRefreshToken: string, newUserInfo: UserInfo) => {
        setAccessToken(newAccessToken);
        setUserInfo(newUserInfo);
        setAuthenticated(true);
        sessionStorage.setItem(ACCESS_TOKEN_KEY, newAccessToken);
        sessionStorage.setItem(REFRESH_TOKEN_KEY, newRefreshToken);
        sessionStorage.setItem(USER_INFO_KEY, JSON.stringify(newUserInfo));
    }, []);

    const logout = useCallback(async () => {
        try {
            // Call your backend API to invalidate the token
            await fetch('/api/auth/logout', {
                method: 'POST',
                headers: { 'Authorization': `Bearer ${accessToken}` }
            });
        } catch (error) {
            console.error('Error during logout:', error);
        } finally {
            setAccessToken(null);
            setUserInfo(null);
            setAuthenticated(false);
            sessionStorage.removeItem(ACCESS_TOKEN_KEY);
            sessionStorage.removeItem(REFRESH_TOKEN_KEY);
            sessionStorage.removeItem(USER_INFO_KEY);
        }
    }, [accessToken]);

    const refreshAuth = useCallback(async (): Promise<boolean> => {
        const refreshToken = sessionStorage.getItem(REFRESH_TOKEN_KEY);
        if (!refreshToken) return false;

        try {
            const response = await fetch('/api/auth/refresh', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ refresh_token: refreshToken })
            });

            if (!response.ok) throw new Error('Failed to refresh token');

            const data = await response.json();
            login(data.access_token, data.refresh_token, data.user_info);
            return true;
        } catch (error) {
            console.error('Error refreshing token:', error);
            await logout();
            return false;
        }
    }, [login, logout]);

    useEffect(() => {
        const storedAccessToken = sessionStorage.getItem(ACCESS_TOKEN_KEY);
        const storedUserInfo = sessionStorage.getItem(USER_INFO_KEY);

        if (storedAccessToken && storedUserInfo) {
            const decodedToken = jwtDecode<DecodedToken>(storedAccessToken);
            if (decodedToken.exp * 1000 > Date.now()) {
                setAccessToken(storedAccessToken);
                setUserInfo(JSON.parse(storedUserInfo));
                setAuthenticated(true);
            } else {
                refreshAuth();
            }
        }
    }, [refreshAuth]);

    return (
        <AuthContext.Provider value={{ accessToken, userInfo, authenticated, login, logout, refreshAuth }}>
            {children}
        </AuthContext.Provider>
    );
};

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};