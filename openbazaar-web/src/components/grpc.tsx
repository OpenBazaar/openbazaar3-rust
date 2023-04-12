"use client"

import { useEffect } from 'react';

const GrpcComponent = () => {
    const handleGrpcCall = async () => {
        const { makeGrpcCall } = await import('../app/grpcClient');
        makeGrpcCall();
    };

    useEffect(() => {
        if (typeof window !== 'undefined') {
            // The gRPC-web client will only be imported and executed on the client-side.
            handleGrpcCall();
        }
    }, []);

    return (<h1></h1>);
};

export default GrpcComponent;