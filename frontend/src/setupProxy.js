const { createProxyMiddleware } = require( "http-proxy-middleware" );

module.exports = function ( app ) {
  app.use(
    "/api", // Set the path to the backend server
    createProxyMiddleware( {
      target: "http://backend:8000", // Point to the backend server
      pathRewrite: { "^/api": "" } // Remove /api from the path
    } )
  );
};
